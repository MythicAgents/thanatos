import { Task, BrowserScriptResponse, TableRowEntry, PlaintextResponse, TableResponse } from "./mythic.js";

type GetEnvResponse = Array<EnvironmentVariable>;

type EnvironmentVariable = {
  key: string,
  value: string,
}

interface EnvironmentVariableRow extends TableRowEntry {
  actions: string
}

function getenv(task: Task, responses: Array<string>): PlaintextResponse | TableResponse<EnvironmentVariableRow> {
  if (task.status == "error") {
    const combined: string = responses.reduce((prev, cur) => {
      return prev + cur;
    }, "");
    return {
      plaintext: combined,
    }
  }

  let rows = [];
  for (let i = 0; i < responses.length; i++) {
    let data: GetEnvResponse = JSON.parse(responses[i]);

    for (let j = 0; j < data.length; j++) {
      let row = {
        "rowStyle": {},
        "actions": {
          "button": {
            "startIcon": "list",
            "name": "Actions",
            "type": "menu",
            "value": [
              {
                "name": "Remove",
                "type": "task",
                "ui_feature": "unsetenv",
                "parameters": JSON.stringify({
                  "variable": data[j].key,
                }),
                "startIcon": "delete",
              },
            ],
          },
        },
        "key": {
          "plaintext": data[j].key,
          "cellStyle": {},
          "copyIcon": true,
        },
        "value": {
          "plaintext": data[j].value,
          "cellStyle": {},
          "copyIcon": true,
        },
      };

      rows.push(row);
    }
  }

  return {
    "table": [{
      "title": "Environment Variables",
      "headers": [
        {
          plaintext: "actions",
          type: "button",
          width: 120,
          disableSort: true
        },
        {
          plaintext: "key",
          type: "string",
          fillWidth: true,
        },
        {
          plaintext: "value",
          type: "string",
          fillWidth: true
        },
      ],
      "rows": rows,
    }]
  };
}
