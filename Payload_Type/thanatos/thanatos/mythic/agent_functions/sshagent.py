from mythic_container.MythicCommandBase import (
    CommandAttributes,
    CommandBase,
    CommandParameter,
    ParameterGroupInfo,
    ParameterType,
    PTTaskCreateTaskingMessageResponse,
    PTTaskMessageAllData,
    SupportedOS,
    TaskArguments,
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
    version = 1
    author = "@M_alphaaa"
    argument_class = SshAgentArguments
    attackmapping = ["T1563.001"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, taskData: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        resp = PTTaskCreateTaskingMessageResponse(TaskID=taskData.Task.ID)
        if taskData.args.get_arg("list"):
            resp.DisplayParams = "-l"
        elif taskData.args.get_arg("disconnect"):
            resp.DisplayParams = "-d"
        elif socket := taskData.args.get_arg("connect"):
            resp.DisplayParams = f"-c {socket}"
        else:
            raise Exception("Invalid arguments")

        return resp
