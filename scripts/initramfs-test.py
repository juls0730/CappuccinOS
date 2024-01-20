import os
import random
import string
import sys

def generate_random_string(length: int):
    return ''.join(random.choices(string.ascii_letters + string.digits, k=length))

def generate_random_data(size: int):
    return os.urandom(size)

def create_random_structure(output_dir: str, depth: int, max_depth: int):
    if depth >= max_depth:
        return

    random_dir = generate_random_string(10)
    os.makedirs(os.path.join(output_dir, random_dir))

    random_file_name = generate_random_string(8)
    random_file_path = os.path.join(output_dir, random_dir, random_file_name)

    with open(random_file_path, 'wb') as file:
        pass

    create_random_structure(os.path.join(output_dir, random_dir), depth + 1, max_depth)

if __name__ == "__main__":
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print("Usage: initramfs-test.py <iteration num> [output_dir]")
        exit(1)

    iterations = int(sys.argv[1])
    output_dir = sys.argv[2] if len(sys.argv) == 3 else "."

    for i in range(iterations):
        max_depth = random.randint(0, 3)
        create_random_structure(output_dir, 0, max_depth)
