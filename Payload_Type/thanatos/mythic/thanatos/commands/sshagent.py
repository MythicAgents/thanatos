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


class SshAgentArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="list",
                type=ParameterType.Boolean,
                cli_name="l",
                description="List agent identities",
                display_name="List agent identities",
                default_value=False,
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=False,
                        ui_position=1,
                    ),
                ],
            ),
            CommandParameter(
                name="connect",
                type=ParameterType.String,
                description="Connect to an ssh agent socket",
                display_name="Connect to an ssh agent socket",
                cli_name="c",
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=False,
                        ui_position=2,
                    ),
                ],
            ),
            CommandParameter(
                name="disconnect",
                type=ParameterType.Boolean,
                description="Disconnect from the ssh agent",
                display_name="Disconnect from the ssh agent",
                cli_name="d",
                default_value=False,
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=False,
                        ui_position=3,
                    ),
                ],
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class SshAgentCommand(CommandBase):
    cmd = "ssh-agent"
    needs_admin = False
    help_cmd = "ssh-agent [-l] [-d] [-c <agent socket path>]"
    description = "List identities and connect to ssh agents"
    version = 2
    author = "@M_alphaaa"
    argument_class = SshAgentArguments
    attackmapping = ["T1563.001"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        if task_data.args.get_arg("list"):
            display_params = "-l"
        elif task_data.args.get_arg("disconnect"):
            display_params = "-d"
        elif socket := task_data.args.get_arg("connect"):
            display_params = f"-c {socket}"
        else:
            return PTTaskCreateTaskingMessageResponse(
                TaskID=task_data.Task.ID,
                Success=False,
                Error="Invalid arguments",
            )

        return PTTaskCreateTaskingMessageResponse(
            TaskID=task_data.Task.ID,
            Success=True,
            DisplayParams=display_params,
        )
