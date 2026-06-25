define({
    "entity",
    id = "base_prop",
    abstract = true,
})

local plants = {
    "Plant0",
    "Plant11",
    "Plant14",
    "Plant15",
    "Plant19",
    "Plant31",
    "Plant36",
    "Plant36B",
    "Plant6",
    "Plant7",
    "Plant9",
    "PlantDecortive1",
    "PlantExotic1",
    "PlantSapling",
}

for i = 1, #plants do
    local name = plants[i]

    define({
        "entity",
        id = name:lower(),
        parent = "base_plant",
        mesh = "models/ss3d/plants/" .. name .. ".glb",
    })
end

local props = {
    "FoamTank",
    "FuelTank",
    "PlatformCart",
    "WaterTank",
    "WaterTankHydro",
}

for i = 1, #props do
    local name = props[i]

    define({
        "entity",
        id = name:lower(),
        parent = "base_prop",
        mesh = "models/ss3d/tanks/" .. name .. ".glb",
    })
end
