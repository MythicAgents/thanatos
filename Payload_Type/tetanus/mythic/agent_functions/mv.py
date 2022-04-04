from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    CommandAttributes,
    SupportedOS
)


class MvArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="source",
                type=ParameterType.String,
                description="Source file or directory to move.",
            ),
            CommandParameter(
                name="destination",
                cli_name="dest",
                type=ParameterType.String,
                description="Destination for move.",
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_locatiojn == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.verify_required_args_have_values()
                args = self.command_line.split(" ")
                self.set_arg("source", args[0])
                self.set_arg("destination", args[1])

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class MvCommand(CommandBase):
    cmd = "mv"
    needs_admin = False
    help_cmd = "mv [source] [destination]"
    description = "Move a file or directory."
    version = 1
    author = "@M_alphaaa"
    argument_class = MvArguments
    attackmapping = ["T1106"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Windows, SupportedOS.Linux ],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        source = task.args.get_arg("source")
        dest = task.args.get_arg("destination")
        task.display_params = f"{source} {dest}"
        return task

    async def process_response(self, response: AgentResponse):
        pass
