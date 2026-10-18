function(task, responses) {
  if (task.status.includes("error")) {
    const combined = responses.reduce((prev, cur) => {
      return prev + cur;
    }, "");
    return { "plaintext": combined };
  }

  let headers = [
    { "plaintext": "actions", "type": "button", "cellStyle": {}, "width": 120, "disableSort": true },
    { "plaintext": "key", "type": "string", "fillWidth": true, "cellStyle": {} },
    { "plaintext": "value", "type": "string", "fillWidth": true, "cellStyle": {} },
  ];

  let rows = [];
  let title = "";

  for (let i = 0; i < responses.length; i++) {
    let data = JSON.parse(responses[i]);

    title = "Environment Variables";

    for (let j = 0; j < data.length; j++) {
      let variable_value = {};
      variable_value[data[j].key] = data[j].value;

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
      "headers": headers,
      "rows": rows,
      "title": title,
    }]
  };
}
