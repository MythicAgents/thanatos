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


class MvArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="source",
                type=ParameterType.String,
                description="Source file or directory to move.",
                display_name="Source file or directory to move.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
            CommandParameter(
                name="destination",
                type=ParameterType.String,
                cli_name="dest",
                description="Destination for move.",
                display_name="Destination for move.",
                parameter_group_info=[ParameterGroupInfo(ui_position=2)],
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
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, taskData: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        source = taskData.args.get_arg("source")
        dest = taskData.args.get_arg("destination")
        return PTTaskCreateTaskingMessageResponse(
            TaskID=taskData.Task.ID, DisplayParams=f"{source} {dest}"
        )
