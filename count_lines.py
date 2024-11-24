import os

def get_files(path):
    files = []
    for item in os.listdir(path):
        if item == 'node_modules' or item == 'public' or item == 'target' or item == '.git':
            continue

        if item == 'Cargo.lock' or item == 'package-lock.json':
            continue

        item_path = os.path.join(path, item)
        if os.path.isfile(item_path):
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
