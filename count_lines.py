import os

def get_files(path):
    files = []
    for item in os.listdir(path):
        item_path = os.path.join(path, item)

        if item == 'node_modules' or item == 'build' or item == 'target':
            continue

        if os.path.isfile(item_path):
            _, ext = os.path.splitext(item)
            if ext != '.py' and ext != '.rs' and ext != '.ts' and ext != '.tsx' and ext != '.js' and ext != '.jsx' and ext != '.css' and ext != '.toml':
                continue

            print(item_path)
            files.append(item_path)
        elif os.path.isdir(item_path):
            files.extend(get_files(item_path))

    return files


files = get_files('.')

line_count = 0
for file in files:
    handle = open(file, 'r')

    lines = handle.readlines()
    line_count += len(lines)

    handle.close()

print(f'lines: {line_count}, pages: {line_count / 25}')
