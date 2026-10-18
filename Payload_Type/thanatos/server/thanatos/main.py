import argparse
import importlib
import pathlib
import sys

from thanatos import config


def try_import_dotenv():
    try:
        return importlib.import_module("dotenv")
    except ModuleNotFoundError:
        return None


def run_service():
    import mythic_container  # noqa # pylint: disable=import-outside-toplevel

    # pylint: disable-next=unused-import, import-outside-toplevel
    from thanatos.payload_type import Thanatos

    mythic_container.start_and_run_forever()


def main(cmdline: list[str] | None = None):
    parser = argparse.ArgumentParser(
        "thanatos-server", formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )

    parser.add_argument(
        "--agent-code-path",
        type=pathlib.Path,
        default=config.AGENT_CODE_PATH,
        help="Path to the root of the agent code",
    )

    dotenv = try_import_dotenv()
    if dotenv is not None:
        parser.add_argument(
            "--dotenv",
            type=pathlib.Path,
            default=pathlib.Path(".env"),
            help="Path to .env file for loading configuration",
        )

    args = parser.parse_args(cmdline)

    if dotenv is not None:
        dotenv.load_dotenv(args.dotenv)

    config.AGENT_CODE_PATH = args.agent_code_path
    run_service()


if __name__ == "__main__":
    main(sys.argv)
