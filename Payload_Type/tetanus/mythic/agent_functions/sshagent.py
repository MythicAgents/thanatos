from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
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
                default_value=False,
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=False,
                        group_name="Default",
                    ),
                ],
            ),
            CommandParameter(
                name="connect",
                type=ParameterType.String,
                description="Connect to an ssh agent socket",
                cli_name="c",
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=False,
                        group_name="Default",
                    ),
                ],
            ),
            CommandParameter(
                name="disconnect",
                type=ParameterType.Boolean,
                description="Disconnect from the ssh agent",
                cli_name="d",
                default_value=False,
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=False,
                        group_name="Default",
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

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        if task.args.get_arg("list"):
            task.display_params = "-l"
        elif task.args.get_arg("disconnect"):
            task.display_params = "-d"
        elif socket := task.args.get_arg("connect"):
            task.display_params = f"-c {socket}"
        else:
            raise Exception("Invalid arguments")

        return task

    async def process_response(self, response: AgentResponse):
        pass
