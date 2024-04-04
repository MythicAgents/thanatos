from mythic_container.MythicCommandBase import (
    BrowserScript,
    TaskArguments,
    CommandBase,
    CommandAttributes,
    SupportedOS,
    PTTaskMessageAllData,
    PTTaskCreateTaskingMessageResponse,
)


class GetEnvArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = []

    async def parse_arguments(self):
        pass


class GetEnvCommand(CommandBase):
    cmd = "getenv"
    needs_admin = False
    help_cmd = "getenv"
    description = "Get all environment variables."
    version = 2
    author = "@M_alphaaa"
    supported_ui_features = ["callback_table:getenv"]
    argument_class = GetEnvArguments
    attackmapping = ["T1082"]
    browser_script = BrowserScript(
        script_name="getenv", author="@M_alphaaa", for_new_ui=True
    )
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        return PTTaskCreateTaskingMessageResponse(TaskID=task_data.Task.ID, Success=True)
