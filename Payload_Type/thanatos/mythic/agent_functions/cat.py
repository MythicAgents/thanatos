from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandParameter,
    CommandBase,
    MythicTask,
    AgentResponse,
    ParameterType,
)


class CatArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="file",
                type=ParameterType.String,
                description="File to display the contents of",
            ),
        ]

    async def parse_arguments(self):
        if (
            self.tasking_location == "command_line"
            or self.tasking_location == "browserscript"
        ):
            self.set_arg("file", self.get_command_line())

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class CatCommand(CommandBase):
    cmd = "cat"
    needs_admin = False
    help_cmd = "cat [file path]"
    description = "Cat a file using rust functions."
    version = 1
    author = "@M_alphaaa"
    supported_ui_features = ["cat"]
    argument_class = CatArguments
    attackmapping = ["T1005", "T1039", "T1025"]

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        task.display_params = str(task.args.get_arg("file"))
        return task

    async def process_response(self, response: AgentResponse):
        pass
