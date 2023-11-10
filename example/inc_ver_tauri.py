import os
import json
import toml

def inc_ver(ver: str) -> str:
    lst = list(map(int, ver.split(".")))
    lst[1] += 1
    return ".".join((str(i) for i in lst))

def main():
    this_path = os.path.dirname(__file__)
    root = os.path.dirname(this_path)

    with open(
        os.path.join(root, "src-tauri", "tauri.conf.json"), "r+", encoding="utf-8"
    ) as f:
        conf_json = json.load(f)
        old = conf_json["package"]["version"]
        conf_json["package"]["version"] = inc_ver(old)
        f.seek(0)
        f.truncate()
        json.dump(conf_json, f, indent=2)

    with open(os.path.join(root, "Cargo.toml"), "r+", encoding="utf-8") as f:
        cargo_toml = toml.load(f)
        old = cargo_toml["workspace"]["package"]["version"]
        cargo_toml["workspace"]["package"]["version"] = inc_ver(old)
        f.seek(0)
        f.truncate()
        toml.dump(cargo_toml, f)

    with open(os.path.join(root, "package.json"), "r+", encoding="utf-8") as f:
        conf_json = json.load(f)
        old = conf_json["version"]
        conf_json["version"] = inc_ver(old)
        f.seek(0)
        f.truncate()
        json.dump(conf_json, f, indent=2)


if __name__ == "__main__":
    main()
    print(f"inc ver success")
