import csv

def remove_duplicates(file_path):
    unique_lines = set()
    temp_file_path = file_path + ".tmp"

    with open(file_path, 'r', newline='', encoding='utf-8') as infile, \
         open(temp_file_path, 'w', newline='', encoding='utf-8') as outfile:
        
        reader = csv.reader(infile)
        writer = csv.writer(outfile)
        
        for row in reader:
            line = ','.join(row)
            if line not in unique_lines:
                writer.writerow(row)
                unique_lines.add(line)

    # Replace the original file with the deduplicated file
    import os
    os.replace(temp_file_path, file_path)

# Call the function to remove duplicates
remove_duplicates('data/data_nl_1.csv')
