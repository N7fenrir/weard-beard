import asyncio
import json
import logging
from typing import Any, Optional
from . import config, state
from .stats import calculate_stats
import aio_pika
import aio_pika.abc

log = logging.getLogger(__name__)

async def consume_messages(shutdown_event: asyncio.Event) -> None:
    """Connects to RabbitMQ and consumes messages from the specified queue."""
    connection: Optional[aio_pika.RobustConnection] = None
    log.info("Starting RabbitMQ consumer task...")

    while not shutdown_event.is_set():
        try:
            log.info(f"Attempting connection to RabbitMQ at {config.RABBITMQ_HOST}:{config.RABBITMQ_PORT}...")
            connection = await aio_pika.connect_robust(
                host=config.RABBITMQ_HOST,
                port=config.RABBITMQ_PORT,
                login=config.RABBITMQ_USER,
                password=config.RABBITMQ_PASS,
                virtualhost=config.RABBITMQ_VHOST,
                timeout=10
            )
            log.info("RabbitMQ connection successful.")

            async with connection:
                channel: aio_pika.abc.AbstractChannel = await connection.channel()
                await channel.set_qos(prefetch_count=10)
                try:
                    queue: aio_pika.abc.AbstractQueue = await channel.declare_queue(
                        config.QUEUE_NAME,
                        durable=True,
                        passive=True
                    )
                except aio_pika.exceptions.ChannelClosedByBroker as e:
                     log.error(f"Queue '{config.QUEUE_NAME}' not found or incompatible on broker: {e}. Check RabbitMQ setup. Retrying connection...")
                     await asyncio.sleep(config.RECONNECT_DELAY_S)
                     continue
                except Exception as e:
                     log.error(f"Failed to declare queue '{config.QUEUE_NAME}': {e}. Retrying connection...")
                     await asyncio.sleep(config.RECONNECT_DELAY_S)
                     continue

                log.info(f"Waiting for messages on queue '{config.QUEUE_NAME}'...")

                async with queue.iterator() as queue_iter:
                    message: aio_pika.abc.AbstractIncomingMessage
                    async for message in queue_iter:
                        if shutdown_event.is_set():
                            log.info("Shutdown signal received, stopping consumer iterator.")
                            break
                        async with message.process(ignore_processed=True): # Auto-ack on success
                            await handle_message(message)

                        await asyncio.sleep(0) # Yield control

        except aio_pika.exceptions.AMQPConnectionError as e:
            log.error(f"RabbitMQ connection error: {e}. Retrying in {config.RECONNECT_DELAY_S} seconds...")
        except asyncio.CancelledError:
            log.info("Consumer task cancelled.")
            break
        except Exception as e:
            log.error(f"Unexpected error in consumer: {e}. Retrying in {config.RECONNECT_DELAY_S} seconds...")
        finally:
            if connection and not connection.is_closed:
                 log.info("Closing RabbitMQ connection.")
                 await connection.close()
            if not shutdown_event.is_set():
                await asyncio.sleep(config.RECONNECT_DELAY_S)

    log.info("RabbitMQ consumer task finished.")


async def handle_message(message: aio_pika.abc.AbstractIncomingMessage) -> None:
    """Processes a single incoming message, calculates stats, and logs."""
    try:
        body: str = message.body.decode()
        data: dict = json.loads(body)

        sensor_id: Optional[str] = data.get("sensor_id")
        sensor_type: Optional[str] = data.get("sensor_type")
        value_any: Any = data.get("value") # Use Any for initial get

        if not sensor_type or not isinstance(sensor_type, str):
            log.warning(f"Received message missing or invalid 'sensor_type': {body[:100]}...")
            return
        if value_any is None or not isinstance(value_any, (int, float)):
            log.warning(f"Received message missing or invalid 'value' for type {sensor_type}: {body[:100]}...")
            return
        if not sensor_id or not isinstance(sensor_id, str):
            log.warning(f"Received message missing or invalid 'sensor_id' for type {sensor_type}: {body[:100]}...")
            sensor_id = "UNKNOWN_ID"

        value: float = float(value_any)

        mean_val: Optional[float] = None
        stdev_val: Optional[float] = None
        window_len: int = 0

        async with state.stats_lock:
            state.initialize_sensor_type(sensor_type)
            stats = state.sensor_stats[sensor_type]
            stats["values"].append(value)
            stats["count"] += 1
            new_calcs = calculate_stats(stats["values"])

            stats["mean"] = new_calcs["mean"]
            stats["stdev"] = new_calcs["stdev"]
            mean_val = stats["mean"]
            stdev_val = stats["stdev"]
            window_len = len(stats["values"])
        current_filter = state.get_log_filter()
        should_log = current_filter is None or current_filter.lower() == sensor_type.lower()

        if should_log:
            mean_str = f"{mean_val:.2f}" if mean_val is not None else "N/A"
            stdev_str = f"{stdev_val:.2f}" if stdev_val is not None else "N/A"
            log.info(
                f"[{sensor_type}] Msg Rcvd (ID: {sensor_id}): Value={value:.2f}. "
                f"Stats (last {window_len}): Mean={mean_str}, StDev={stdev_str}"
            )
    except json.JSONDecodeError:
        log.warning(f"Received non-JSON message: {message.body[:100]}...")
    except Exception as e:
        log.error(f"Error processing message: {e}", exc_info=True)