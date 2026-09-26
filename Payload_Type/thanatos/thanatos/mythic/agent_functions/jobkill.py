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


class JobKillArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="id",
                type=ParameterType.Number,
                description="Job id of the job to kill.",
                display_name="Job id of the job to kill.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.set_arg("id", int(self.command_line))

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class JobKillCommand(CommandBase):
    cmd = "jobkill"
    needs_admin = False
    help_cmd = "jobkill [job id]"
    description = "Kill a job with the specified ID."
    version = 1
    author = "@M_alphaaa"
    argument_class = JobKillArguments
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, taskData: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        return PTTaskCreateTaskingMessageResponse(
            TaskID=taskData.Task.ID, DisplayParams=str(taskData.args.get_arg("id"))
        )
