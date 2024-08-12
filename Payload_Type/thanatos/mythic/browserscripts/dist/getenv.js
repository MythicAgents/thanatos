"use strict";
function getenv(task, responses) {
    if (task.status.includes("error")) {
        var combined = responses.reduce(function (prev, cur) {
            return prev + cur;
        }, "");
        return { "plaintext": combined };
    }
    var headers = [
        { "plaintext": "actions", "type": "button", "cellStyle": {}, "width": 120, "disableSort": true },
        { "plaintext": "key", "type": "string", "fillWidth": true, "cellStyle": {} },
        { "plaintext": "value", "type": "string", "fillWidth": true, "cellStyle": {} },
    ];
    var rows = [];
    var title = "";
    for (var i = 0; i < responses.length; i++) {
        var data = JSON.parse(responses[i]);
        title = "Environment Variables";
        for (var j = 0; j < data.length; j++) {
            var variable_value = {};
            variable_value[data[j].key] = data[j].value;
            var row = {
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
                "headers": headers,
                "rows": rows,
                "title": title,
            }]
    };
}
