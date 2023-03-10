from pcbnew import *
import pcbnew
import json

json_file_path = "./pcb/mount_locations.json"
pcb_file_path = "/home/brooks/github/warning-light/pcb/warning-light/warning-light.kicad_pcb"

OFFSET_X = 50
OFFSET_Y = 50

WIDTH = 80
HEIGHT = 80

class MountingHole():
    def __init__(self, footprint: FOOTPRINT):
        self.footprint = footprint

    @staticmethod
    def is_mounting_hole(footprint: FOOTPRINT):
        naming_convention = "MountingHole"

        text = footprint.Value().GetText()

        if naming_convention in text:
            print(f"{text} DOES match key mounting hole")
            return True
        else:
            print(f"{text} does not match mounting hole")
            return False

    def __repr__(self):
        return f"mounting hole"

def read_hole_locations() -> list[list[float]]:

    with open(json_file_path, "r") as f:
        json_data = json.load(f)
        return json_data["holes"]

def place_holes(hole_locations: list[list[float]], hole_footprints: list[MountingHole]):
    assert(len(hole_footprints) == len(hole_locations));

    for (location, footprint) in zip(hole_locations, hole_footprints):
        raw_x, raw_y = location

        x = FromMM(OFFSET_X + raw_x + WIDTH / 2)
        y = FromMM(OFFSET_Y - raw_y + HEIGHT / 2)

        footprint.footprint.SetX(x)
        footprint.footprint.SetY(y)

def main():
    pcb = LoadBoard(pcb_file_path)
    footprints = pcb.GetFootprints()

    hole_footprints = []

    for footprint in footprints:
        if MountingHole.is_mounting_hole(footprint):
            hole_footprints.append(MountingHole(footprint))

    hole_data = read_hole_locations()

    place_holes(hole_data, hole_footprints)


class MountHoleLayout(ActionPlugin):
    def defaults(self):
        self.name = "Mount hole layout plugin"
        self.category = "warning light utils"
        self.description = "automatically lays out hole locations according to cad specification"

    def Run(self):
        main()

#main()
MountHoleLayout().register()
