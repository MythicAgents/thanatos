from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    SupportedOS,
    PTTaskMessageAllData,
    PTTaskCreateTaskingMessageResponse,
)

# TODO: Refactor implementation


class JobsArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = []

    async def parse_arguments(self):
        pass


class JobsCommand(CommandBase):
    cmd = "jobs"
    needs_admin = False
    help_cmd = "jobs"
    description = "List running background jobs."
    version = 2
    author = "@M_alphaaa"
    supported_ui_features = ["callback_table:jobs"]
    argument_class = JobsArguments
    attackmapping = []
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        return PTTaskCreateTaskingMessageResponse(
            TaskID=task_data.Task.ID,
            Success=True,
        )
