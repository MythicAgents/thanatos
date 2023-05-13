from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    SupportedOS,
    MythicTask,
    PTTaskMessageAllData,
    PTTaskProcessResponseMessageResponse,
)


class GetPrivsArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = []

    async def parse_arguments(self):
        pass


class GetPrivsCommand(CommandBase):
    cmd = "getprivs"
    needs_admin = False
    help_cmd = "getprivs"
    description = "Get current user privileges."
    version = 1
    author = "@M_alphaaa"
    supported_ui_features = ["callback_table:getprivs"]
    argument_class = GetPrivsArguments
    attackmapping = ["T1078"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        return task

    async def process_response(
        self, task: PTTaskMessageAllData, response: str
    ) -> PTTaskProcessResponseMessageResponse:
        pass
