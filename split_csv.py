import os
import csv

def split_csv(input_file, output_folder, max_lines=250000):
    # Ensure the output directory exists
    if not os.path.exists(output_folder):
        print(f"Output folder '{output_folder}' does not exist. Creating it.")
        os.makedirs(output_folder)
    else:
        print(f"Output folder '{output_folder}' already exists.")

    with open(input_file, 'r', newline='', encoding='utf-8') as csvfile:
        print(f"Opened input file '{input_file}' for reading.")
        reader = csv.reader(csvfile)
        headers = next(reader)  # Read the headers
        print(f"Read headers: {headers}")

        file_count = 0
        current_line_count = 0
        current_file = None
        writer = None

        for row in reader:
            if current_line_count == 0:
                # Close the previous file if it exists
                if current_file:
                    print(f"Closing file '{current_file.name}'.")
                    current_file.close()

                # Open a new file
                file_count += 1
                current_file_path = os.path.join(output_folder, f'part_{file_count}.csv')
                print(f"Opening new file '{current_file_path}' for writing.")
                current_file = open(current_file_path, 'w', newline='', encoding='utf-8')
                writer = csv.writer(current_file)

                # Write the headers to the new file
                writer.writerow(headers)
                print(f"Wrote headers to '{current_file_path}'.")

            # Write the current row
            writer.writerow(row)
            current_line_count += 1
            print(f"Wrote row {current_line_count} to '{current_file_path}'.")

            # If the current file has reached the max lines, reset the line count
            if current_line_count >= max_lines:
                print(f"Reached max lines for '{current_file_path}'. Resetting line count.")
                current_line_count = 0

        # Close the last file if it was open
        if current_file:
            print(f"Closing last file '{current_file.name}'.")
            current_file.close()

# Example usage
print("Starting CSV split operation.")
split_csv('adressen.csv', './data_split')
print("CSV split operation completed.")
