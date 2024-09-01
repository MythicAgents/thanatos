import ipaddress
from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    SupportedOS,
    PTTaskMessageAllData,
    PTTaskCreateTaskingMessageResponse,
)


class PortScanArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="hosts",
                type=ParameterType.Array,
                description="List of hosts or subnets to scan",
                display_name="List of hosts or subnets to scan",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
            CommandParameter(
                name="ports",
                type=ParameterType.String,
                description=(
                    "List of ports to scan. "
                    "Can use the dash character to specify a range."
                ),
                display_name=(
                    "List of ports to scan. "
                    "Can use the dash character to specify a range."
                ),
                parameter_group_info=[ParameterGroupInfo(ui_position=2)],
            ),
            CommandParameter(
                name="interval",
                type=ParameterType.Number,
                description=(
                    "Interval in milli-seconds to sleep before scanning a new port/host."
                ),
                default_value=1000,
                parameter_group_info=[ParameterGroupInfo(ui_position=3)],
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class PortScanCommand(CommandBase):
    cmd = "portscan"
    needs_admin = False
    help_cmd = "portscan [popup]"
    description = "Scan host(s) for open ports."
    version = 2
    author = "@M_alphaaa"
    argument_class = PortScanArguments
    attackmapping = ["T1046"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        ports = task_data.args.get_arg("ports")
        task_data.args.set_arg("ports", ports.replace(" ", ""))

        ipaddrs = task_data.args.get_arg("hosts")
        for ip in ipaddrs:
            if "/" in ip:
                try:
                    ipaddress.ip_network(ip)
                except ValueError:
                    return PTTaskCreateTaskingMessageResponse(
                        TaskID=task_data.Task.ID, Success=False, Error="Invalid IP subnet"
                    )
            else:
                try:
                    ipaddress.ip_address(ip)
                except ValueError:
                    return PTTaskCreateTaskingMessageResponse(
                        TaskID=task_data.Task.ID,
                        Success=False,
                        Error="Invalid IP address",
                    )

        return PTTaskCreateTaskingMessageResponse(
            TaskID=task_data.Task.ID,
            Success=True,
        )
