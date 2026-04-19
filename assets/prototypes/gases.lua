-- gases
-- NOTE: gas ids are given by define order
-- I think this is actualy an anti-pattern, 
-- and should be looked into better
define {
    "gas",
    id = "oxygen",
    molar_heat_capacity = 21.1,
}

define {
    "gas",
    id = "nitrogen",
    molar_heat_capacity = 20.7,
}

define {
    "gas",
    id = "carbon_dioxide",
    molar_heat_capacity = 28.4,
}

define {
    "gas",
    id = "nitrogen_dioxide",
    molar_heat_capacity = 30.0,
}

define {
    "gas",
    id = "plasma",
    molar_heat_capacity = 300.0,
}

-- mixtures

define {
    "gas_mixture",
    id = "air",
    pressure = 101.325,
    temperature = 295,
    ratios = {
        oxygen = 0.21,
        nitrogen = 0.79,
    }
}

