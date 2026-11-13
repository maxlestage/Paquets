{
  "name": "{{title}}",
  "description": "A full-stack Rust app built with Paquets.",
  "keywords": ["rust", "paquets", "web"],
  "stack": "container",
  "success_url": "/",
  "env": {
    "NUAGE_URL": {
      "description": "Address of your Nuage DB server (https://…). Without it, the data is kept on the dyno's disk and lost at each restart.",
      "required": false
    },
    "NUAGE_TOKEN": {
      "description": "Token of your Nuage DB server (its NUAGE_TOKEN).",
      "required": false
    }
  }
}
