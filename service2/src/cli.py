import asyncio
import json
import logging
import sys
from typing import Any, Dict, Optional
from . import config, state

log = logging.getLogger(__name__)

async def handle_cli_input(shutdown_event: asyncio.Event) -> None:
    """Handles user input from stdin asynchronously."""
    log.info("CLI handler started. Type 'help' for commands.")
    loop = asyncio.get_running_loop()
    reader = asyncio.StreamReader()
    protocol = asyncio.StreamReaderProtocol(reader)

    try:
        # Connect stdin for reading commands asynchronously
        await loop.connect_read_pipe(lambda: protocol, sys.stdin)
    except (NotImplementedError, OSError) as e:
        log.warning(f"Async stdin reading not supported or failed: {e}. CLI commands unavailable.")
        await shutdown_event.wait() # Keep task alive but non-functional
        return

    while not shutdown_event.is_set():
        sys.stdout.write("> ") # Display prompt
        sys.stdout.flush()
        try:
            res = await asyncio.wait_for(reader.readline(), timeout=config.CLI_CHECK_INTERVAL_S)
            line = res.decode().strip()

            if not line:
                if reader.at_eof():
                     log.info("EOF detected on stdin. Shutting down.")
                     shutdown_event.set()
                     break
                continue

            parts = line.lower().split(maxsplit=1)
            command: str = parts[0]
            args: str = parts[1] if len(parts) > 1 else ""

            # --- Command Processing ---
            if command == "filter":
                new_filter: Optional[str] = None
                if not args or args == "all":
                    log.info("Log filter cleared. Showing all sensor types.")
                    new_filter = None
                else:
                    new_filter = args.capitalize()
                    log.info(f"Log filter set to: {new_filter}")
                state.set_log_filter(new_filter)

            elif command == "export":
                filename: str = args if args else "sensor_stats_export.json"
                await export_stats(filename)

            elif command == "stats":
                 await print_current_stats()

            elif command == "help":
                 print_help()

            elif command == "quit" or command == "exit":
                log.info("Quit command received. Initiating shutdown.")
                shutdown_event.set()
                break

            else:
                log.warning(f"Unknown command: '{command}'. Type 'help'.")

        except asyncio.TimeoutError:
             await asyncio.sleep(0)
             continue
        except asyncio.CancelledError:
            log.info("CLI handler cancelled.")
            break
        except Exception as e:
             log.error(f"Error in CLI handler: {e}", exc_info=True)
             await asyncio.sleep(1)

    log.info("CLI handler finished.")

def print_help() -> None:
     """Prints available CLI commands."""
     print("\nAvailable Commands:")
     print("  filter <type>  - Show logs only for the specified sensor type (e.g., filter Temperature)")
     print("  filter all     - Show logs for all sensor types (default)")
     print("  stats          - Print current statistics for all sensor types")
     print("  export <file>  - Export current statistics to a JSON file (default: sensor_stats_export.json)")
     print("  quit / exit    - Stop the service")
     print("  help           - Show this help message\n")

async def export_stats(filename: str) -> None:
    """Exports the current sensor statistics to a JSON file."""
    log.info(f"Exporting statistics to '{filename}'...")
    async with state.stats_lock:
        export_data: Dict[str, Dict[str, Any]] = {}
        for sensor_type, data in state.sensor_stats.items():
            export_data[sensor_type] = {
                "values": list(data["values"]),
                "mean": data["mean"],
                "stdev": data["stdev"],
                "count": data["count"]
            }

    try:
        with open(filename, "w") as f:
            json.dump(export_data, f, indent=2)
        log.info(f"Successfully exported stats to '{filename}'")
    except IOError as e:
        log.error(f"Failed to write export file '{filename}': {e}")
    except Exception as e:
        log.error(f"An unexpected error occurred during export: {e}", exc_info=True)

async def print_current_stats() -> None:
     """Prints the current calculated statistics."""
     print("\n--- Current Sensor Statistics ---")
     async with state.stats_lock:
        if not state.sensor_stats:
             print("  No statistics calculated yet.")
             return

        # Sort by type for consistent output
        sorted_types = sorted(state.sensor_stats.keys())

        for sensor_type in sorted_types:
             data = state.sensor_stats[sensor_type]
             mean_str = f"{data['mean']:.2f}" if data["mean"] is not None else "N/A"
             stdev_str = f"{data['stdev']:.2f}" if data["stdev"] is not None else "N/A"
             print(f"  {sensor_type}:")
             print(f"    Count: {data['count']}")
             print(f"    Window (last {len(data['values'])}): {list(data['values'])}")
             print(f"    Mean:  {mean_str}")
             print(f"    StDev: {stdev_str}")
     print("---------------------------------\n")