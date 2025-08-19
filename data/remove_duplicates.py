import glob
import pandas as pd

def remove_duplicates(input_file, output_file, subset=None, keep='first'):
    df = pd.read_csv(input_file)
    
    df_clean = df.drop_duplicates(subset=subset, keep=keep)
    
    df_clean.to_csv(output_file, index=False)
    
    print(f"Original entries: {len(df)}")
    print(f"Unique entries: {len(df_clean)}")
    print(f"Removed {len(df) - len(df_clean)} duplicates")
    print(f"Cleaned data saved to {output_file}")

csv_files = glob.glob('*.csv')
remove_duplicates("2025-08-12_shggzy.csv", "remove_dup_2025-08-12_shggzy.csv")

# print("Found CSV files:")
# for file in csv_files:
#    remove_duplicates(file, 'remove_dup' + file)


