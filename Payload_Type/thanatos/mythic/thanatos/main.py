#!/usr/bin/env python3
import mythic_container
import thanatos.builder as _
import thanatos.commands as _


def main():
    mythic_container.mythic_service.start_and_run_forever()


if __name__ == "__main__":
    main()
