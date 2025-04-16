import asyncio
import logging
import signal
from src.helpers import config, rabbitmq, cli
from typing import Set

log = logging.getLogger(__name__)

async def signal_handler(sig: signal.Signals, shutdown_event: asyncio.Event) -> None:
    """Handles OS signals for graceful shutdown."""
    log.info(f"Received signal {sig.name}. Initiating shutdown...")
    if not shutdown_event.is_set():
        shutdown_event.set()

async def main() -> None:
    """Main asynchronous entry point."""
    logging.basicConfig(
        level=logging.INFO,
        format=config.LOG_FORMAT,
        datefmt=config.LOG_DATE_FORMAT,
    )
    log.info("Starting Sensor Analyzer Service (Service 2)...")
    log.info(f"Connecting to RabbitMQ: {config.RABBITMQ_HOST}:{config.RABBITMQ_PORT}, Queue: {config.QUEUE_NAME}")

    shutdown_event = asyncio.Event()

    loop = asyncio.get_running_loop()
    for sig_name in ('SIGINT', 'SIGTERM'):
        if hasattr(signal, sig_name):
            sig = getattr(signal, sig_name)
            try:
                loop.add_signal_handler(
                    sig,
                    lambda s=sig: asyncio.create_task(signal_handler(s, shutdown_event))
                )
                log.debug(f"Signal handler added for {sig.name}")
            except NotImplementedError:
                log.warning(f"Signal handler for {sig_name} not supported on this platform.")


    consumer_task = asyncio.create_task(rabbitmq.consume_messages(shutdown_event))
    cli_task = asyncio.create_task(cli.handle_cli_input(shutdown_event))
    tasks: Set[asyncio.Task] = {consumer_task, cli_task}

    await shutdown_event.wait()

    log.info("Shutdown signal received. Cancelling running tasks...")

    # Attempt graceful cancellation
    for task in tasks:
        if not task.done():
            task.cancel()

    results = await asyncio.gather(*tasks, return_exceptions=True)

    for i, result in enumerate(results):
        if isinstance(result, Exception) and not isinstance(result, asyncio.CancelledError):
            task_name = tasks.pop().get_name() # Get corresponding task name (approximate)
            log.error(f"Task {task_name} raised an exception: {result}", exc_info=result)

    log.info("Sensor Analyzer Service stopped.")

def start():
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        log.info("Service interrupted by user (KeyboardInterrupt).")
