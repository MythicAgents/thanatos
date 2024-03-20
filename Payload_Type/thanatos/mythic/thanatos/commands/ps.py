from mythic_container.MythicCommandBase import (
    BrowserScript,
    TaskArguments,
    CommandBase,
    CommandAttributes,
    SupportedOS,
    PTTaskMessageAllData,
    PTTaskCreateTaskingMessageResponse,
)

# TODO: Refactor implementation


class PsArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = []

    async def parse_arguments(self):
        pass


class PsCommand(CommandBase):
    cmd = "ps"
    needs_admin = False
    help_cmd = "ps"
    description = "Get a process listing"
    version = 2
    is_process_list = True
    author = "@M_alphaaa"
    supported_ui_features = ["process_browser:list", "callback_table:ps"]
    argument_class = PsArguments
    attackmapping = ["T1106"]
    browser_script = BrowserScript(script_name="ps", author="@M_alphaaa", for_new_ui=True)
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        return PTTaskCreateTaskingMessageResponse(TaskID=task_data.Task.ID, Success=True)
