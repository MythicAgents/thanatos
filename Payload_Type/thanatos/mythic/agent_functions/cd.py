from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandParameter,
    CommandBase,
    MythicTask,
    AgentResponse,
    ParameterType,
)


class CdArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="directory",
                type=ParameterType.String,
                description="Directory to cd to.",
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            self.set_arg("directory", self.get_command_line())

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class CdCommand(CommandBase):
    cmd = "cd"
    needs_admin = False
    help_cmd = "cd [directory]"
    description = "Change working directory (can be relative, but no tilde/~)."
    version = 1
    author = "@M_alphaaa"
    argument_class = CdArguments
    attackmapping = ["T1083"]

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        task.display_params = str(task.args.get_arg("directory"))
        return task

    async def process_response(self, response: AgentResponse):
        pass
