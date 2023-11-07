import os
import json
import toml

version = "0.3.0"


def main():
    this_path = os.path.dirname(__file__)
    root = os.path.dirname(this_path)
    with open(
        os.path.join(root, "src-tauri", "tauri.conf.json"), "r+", encoding="utf-8"
    ) as f:
        conf_json = json.load(f)
        conf_json["package"]["version"] = version
        f.seek(0)
        f.truncate()
        json.dump(conf_json, f, indent=2)

    with open(os.path.join(root, "Cargo.toml"), "r+", encoding="utf-8") as f:
        cargo_toml = toml.load(f)
        cargo_toml["workspace"]["package"]["version"] = version
        f.seek(0)
        f.truncate()
        toml.dump(cargo_toml, f)


if __name__ == "__main__":
    main()
    print(f"inc ver to {version}")
