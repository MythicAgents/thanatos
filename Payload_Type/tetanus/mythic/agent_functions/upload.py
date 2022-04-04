from mythic_payloadtype_container.MythicRPC import MythicRPC
from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandParameter,
    CommandBase,
    AgentResponse,
    MythicTask,
    ParameterType,
    MythicStatus,
    CommandAttributes,
    SupportedOS
)
import json
import sys
import base64


class UploadArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="file",
                type=ParameterType.File,
                description="file to upload",
            ),
            CommandParameter(
                name="path (including name)",
                type=ParameterType.String,
                description="Path where to upload the file including the file name.",
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class UploadCommand(CommandBase):
    cmd = "upload"
    needs_admin = False
    help_cmd = "upload"
    description = (
        "Upload a file to the target machine by selecting a file from your computer."
    )
    version = 1
    is_file_upload = True
    supported_ui_features = ["file_browser:upload"]
    author = "@M_alphaaa"
    attackmapping = ["T1030", "T1105", "T1132"]
    argument_class = UploadArguments
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Windows, SupportedOS.Linux ],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        try:
            original_file_name = json.loads(task.original_params)["file"]
            if len(task.args.get_arg("path")) == 0:
                task.args.add_arg("path", original_file_name)
            elif task.args.get_arg("path")[-1] == "/":
                task.args.add_arg("path", task.args.get_arg("path") + original_file_name)
            file_resp = await MythicRPC().execute(
                "create_file",
                task_id=task.id,
                file=base64.b64encode(task.args.get_arg("file").encode()).decode(),
                saved_file_name=original_file_name,
                delete_after_fetch=True,
            )
            if file_resp.status == MythicStatus.Success:
                task.args.add_arg("file", file_resp.response["agent_file_id"])
                task.display_params = (
                    f"{original_file_name} to {task.args.get_arg('path')}"
                )
            else:
                raise Exception("Error from Mythic: " + str(file_resp.error))
        except Exception as e:
            raise Exception(
                "Error from Mythic: " + str(sys.exc_info()[-1].tb_lineno) + str(e)
            )
        return task

    async def process_response(self, response: AgentResponse):
        pass
