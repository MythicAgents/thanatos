from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    CommandAttributes,
    SupportedOS
)


class RedirectArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="bindhost",
                display_name="Bind Host",
                type=ParameterType.String,
                description="Bind host address",
                default_value="0.0.0.0",
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=True,
                        ui_position=1,
                    )
                ],
            ),
            CommandParameter(
                name="bindport",
                display_name="Bind Port",
                type=ParameterType.Number,
                description="Bind port",
                default_value=8080,
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=True,
                        ui_position=2,
                    )
                ],
            ),
            CommandParameter(
                name="connecthost",
                display_name="Connect Host",
                type=ParameterType.String,
                description="Connect host address",
                default_value="127.0.0.1",
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=True,
                        ui_position=3,
                    )
                ],
            ),
            CommandParameter(
                name="connectport",
                display_name="Connect Port",
                type=ParameterType.Number,
                description="Connect port",
                default_value=80,
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=True,
                        ui_position=4,
                    )
                ],
            ),
        ]

    async def parse_arguments(self):
        if len(self.command_line) > 0:
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                connection = self.command_line.split(":")
                if len(connection) == 4:
                    bindhost = connection[0]
                    try:
                        bindport = int(connection[1])
                    except Exception:
                        raise Exception(f"Invalid bind port {connection[1]}")
                    connecthost = connection[2]
                    try:
                        connectport = int(connection[3])
                    except Exception:
                        raise Exception(f"Invalid connect port {connection[3]}")

                elif len(connection) == 3:
                    bindhost = "0.0.0.0"
                    try:
                        bindport = int(connection[0])
                    except Exception:
                        raise Exception(f"Invalid bind port {connection[0]}")
                    connecthost = connection[1]
                    try:
                        connectport = int(connection[2])
                    except Exception:
                        raise Exception(f"Invalid connect port {connection[2]}")
                else:
                    raise Exception("Invalid relay connection string")

                self.set_arg("bindhost", bindhost)
                self.set_arg("bindport", bindport)
                self.set_arg("connecthost", connecthost)
                self.set_arg("connectport", connectport)
        else:
            raise ValueError("Failed to parse arguments")

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class RedirectCommand(CommandBase):
    cmd = "redirect"
    needs_admin = False
    help_cmd = "redirect -bindhost [host] -bindport [port] -connecthost [host] -connectport [port]"
    description = "Set up a TCP redirector on the machine."
    version = 1
    author = "@M_alphaaa"
    argument_class = RedirectArguments
    attackmapping = ["T1090"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Windows, SupportedOS.Linux ],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        bindhost = task.args.get_arg("bindhost")
        bindport = task.args.get_arg("bindport")
        connecthost = task.args.get_arg("connecthost")
        connectport = task.args.get_arg("connectport")

        task.display_params = f"{bindhost}:{bindport} => {connecthost}:{connectport}"
        return task

    async def process_response(self, response: AgentResponse):
        pass
