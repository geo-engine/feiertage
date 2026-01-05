"""
Append holiday data for the year after next to existing CSV files for HE and NRW.
"""

import sys
import pandas as pd

year_to_add = pd.Timestamp.now().year + 2
states = {"hessen": "HE", "nordrhein-westfalen": "NRW"}

for state, abbrev in states.items():
    # Download holiday data from arbeitstage.org
    url = f"https://www.arbeitstage.org/{state}/feiertage-{year_to_add}-{state}/"
    print(
        f"Downloading holidays for {state} for year {year_to_add} from {url}…",
        file=sys.stderr,
    )
    df = pd.read_html(url)[0]

    # Append to existing CSV file
    with open(f"input/{abbrev}.csv", "a", encoding="utf-8") as f:
        df.to_csv(
            f,
            index=False,
            header=False,
            columns=["Feiertag", "Datum"],
            sep="\t",
        )

    print(f"Appended holidays for {year_to_add} to input/{state}.csv", file=sys.stderr)
