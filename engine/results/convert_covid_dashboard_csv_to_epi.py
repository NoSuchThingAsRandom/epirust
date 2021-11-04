import pandas as pd
from datetime import date

if __name__ == "__main__":
    filename = "real_world_york/york_actual_cases.csv"
    data_frame = pd.read_csv(filename)
    start_date = "2020-01-30"
    start_date = date.fromisoformat(start_date)
    data_frame["hour"] = data_frame.apply(lambda row: (date.fromisoformat(row["date"]) - start_date).days * 24, axis=1)
    # Invert the order of rows
    data_frame=data_frame.reindex(index=data_frame.index[::-1])
    data_frame["susceptible"] = data_frame.apply(lambda row: 198051 - row["cumCasesBySpecimenDate"], axis=1)
    data_frame=data_frame.rename(columns={"newCasesBySpecimenDate": "infected", "cumDeaths28DaysByDeathDate": "deceased"})
    print(data_frame["infected"])
    data_frame.drop(["areaCode", "areaName", "areaType", "date","cumCasesBySpecimenDate"], inplace=True, axis=1)
    data_frame["deceased"]=data_frame["deceased"].fillna(0)
    data_frame["exposed"]=0
    data_frame["hospitalized"]=0
    data_frame["recovered"]=0
    data_frame=data_frame[["hour","susceptible","exposed","infected","hospitalized","recovered","deceased"]]
    print(data_frame["infected"])


    # Duplicate rows
    temp=[]
    base_hour=0
    for row in data_frame.itertuples():

        for hour in range(0,24):
            new_row=[]
            new_row.append(base_hour)
            new_row.append(row[2])
            new_row.append(row[3])
            new_row.append(row[4])
            new_row.append(row[5])
            new_row.append(row[6])
            new_row.append(row[7])
            #new_row.append(row.infected)
            #new_row.append(row.hospitalized)
            #new_row.append(row.recovered)
            #new_row.append(row.deceased)
            temp.append(new_row)
            base_hour+=1
    temp_df=pd.DataFrame(temp,columns=data_frame.columns)
    temp_df.to_csv("york_formatted_actual_cases.csv",index=False)