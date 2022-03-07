from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
)


class MkdirArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="directory",
                cli_name="dir",
                type=ParameterType.String,
                description="Directory to make.",
            )
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.verify_required_args_have_values()
                args = self.command_line
                self.set_arg("directory", args)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class MkdirCommand(CommandBase):
    cmd = "mkdir"
    needs_admin = False
    help_cmd = "mkdir [directory]"
    description = "Make a new directory."
    version = 1
    author = "@M_alphaaa"
    argument_class = MkdirArguments
    attackmapping = ["T1106"]

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        directory = task.args.get_arg("directory")
        task.display_params = directory
        return task

    async def process_response(self, response: AgentResponse):
        pass
