import os

errors = (0, [])
for f in os.listdir("./tests"):
    file = os.path.join("./tests", f)
    if os.path.isfile(file):
        return_code = os.system(f"cargo run {file}")
        if return_code != 0:
            errors = (errors[0] + 1, file)
    else:
        print(f"[TESTS]: unknown file `{file}`")
        exit(1)
if errors[0] == 0:
    print("[TESTS]: successfully ran all tests in `./tests`")
else:
    print(f"[TESTS]: failed with `{errors[0]}` error(s) in `{errors[1]}`")


