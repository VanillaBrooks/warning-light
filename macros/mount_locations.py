import json

OFFSET_CENTER = 35

#(x,y) locations
# -x,y
hole_1 = [-13, 32]
# -x,-y
hole_2 = [-OFFSET_CENTER, -OFFSET_CENTER]
# x,-y
hole_3 = [OFFSET_CENTER, -OFFSET_CENTER]
# x,y
hole_4 = [OFFSET_CENTER, OFFSET_CENTER]

hole_json = {
    "holes": [hole_1, hole_2, hole_3, hole_4],
}

# print it as a CSV
for (i, hole) in enumerate([hole_1, hole_2, hole_3, hole_4]):
    print(f"hole_{i} x,{hole[0]}")
    print(f"hole_{i} y,{hole[1]}")

with open("./pcb/mount_locations.json", "w") as f:
    json.dump(hole_json, f,indent=4)
