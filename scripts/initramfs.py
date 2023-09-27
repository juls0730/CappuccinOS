import os
import gzip
import shutil
import sys

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: python scripts/initramfs.py /path/to/source/directory /path/to/output/directory/initramfs.gz")
        sys.exit(1)

    source_dir, output_file = sys.argv[1], sys.argv[2]

    try:
        with gzip.open(output_file, 'wb') as gz_file:
            for foldername, subfolders, filenames in os.walk(source_dir):
                for filename in filenames:
                    file_path = os.path.join(foldername, filename)
                    rel_path = os.path.relpath(file_path, source_dir)
                    gz_file.write(os.path.join(rel_path).encode())
                    with open(file_path, 'rb') as source_file:
                        shutil.copyfileobj(source_file, gz_file)
        print(f"Compression completed. Output file: {output_file}")
    except Exception as e:
        print(f"Error compressing directory: {str(e)}")
        sys.exit(1)
