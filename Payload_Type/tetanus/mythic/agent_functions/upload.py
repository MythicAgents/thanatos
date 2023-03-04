from mythic_payloadtype_container.MythicRPC import MythicRPC
from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandParameter,
    ParameterGroupInfo,
    CommandBase,
    AgentResponse,
    MythicTask,
    ParameterType,
    MythicStatus,
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
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=1,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="path",
                display_name="path (including name)",
                cli_name="path",
                type=ParameterType.String,
                description="Path where to upload the file including the file name.",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=2,
                        required=True,
                    ),
                ],
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

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        try:
            file_id = task.args.get_arg("file")
            resp = await MythicRPC().execute(
                "get_file", task_id=task.id, file_id=file_id, get_contents=False
            )

            if resp.status != MythicStatus.Success:
                raise Exception("Failed to fetch file: {}".format(resp.error))

            file_name = resp.response[0]["filename"]

            if len(task.args.get_arg("path")) == 0:
                task.args.add_arg("path", file_name)
            elif task.args.get_arg("path")[-1] == "/":
                task.args.add_arg("path", task.args.get_arg("path") + file_name)

            task.display_params = f"{file_name} to {task.args.get_arg('path')}"
            return task
        except Exception as e:
            raise Exception(
                "Error from line {}: {}".format(
                    str(sys.exc_info()[-1].tb_lineno), str(e)
                )
            )

    async def process_response(self, response: AgentResponse):
        pass
