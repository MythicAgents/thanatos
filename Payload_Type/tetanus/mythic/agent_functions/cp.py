from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandParameter,
    CommandBase,
    MythicTask,
    AgentResponse,
    ParameterType,
    CommandAttributes,
    SupportedOS
)


class CpArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="source",
                type=ParameterType.String,
                description="Source file to copy.",
            ),
            CommandParameter(
                name="destination",
                cli_name="dest",
                display_name="destination",
                type=ParameterType.String,
                description="Destination for copied source file.",
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.verify_required_args_have_values()
                args = self.command_line.split(" ")
                self.set_arg("source", args[0])
                self.set_arg("destination", args[1])

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class CpCommand(CommandBase):
    cmd = "cp"
    needs_admin = False
    help_cmd = "cp [source] [destination]"
    description = "Copy a file from one location to another."
    version = 1
    author = "@M_alphaaa"
    argument_class = CpArguments
    attackmapping = ["T1570"]
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
