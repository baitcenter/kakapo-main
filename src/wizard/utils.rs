
use rand::Rng;
use rand;

pub fn random_parrot_name() -> &'static str {
    //From https://en.wikipedia.org/wiki/List_of_kakapo
    let parrot_names = [
        "Adelaide", "Alice", "Aparima", "Aranga", "Atareta",
        "Aumaria", "Awarua", "Bella", "Boomer", "Cyndy",
        "Dusky", "Esperance", "Evohe", "Flossie", "Gertrude",
        "Hakatere", "Hananui", "Hauturu", "Heather", "Hera",
        "Hinemoa", "Hine", "Hoki", "Huhana", "Ihi", "Jean",
        "JEM", "Jemma", "Konini", "Kuia", "Kuihi", "Kura",
        "Lisa", "Mahli", "Makoera", "Marama", "Margaret",
        "Marian", "Mila", "Millie", "Mukeke", "Nora", "Ninihi",
        "Pearl", "Pounamu", "Punga", "Pura", "Queenie", "Ra",
        "Rakiura", "Rimu", "Roha", "Ruth", "Solstice", "Stella",
        "Sue", "Suzanne", "Tia", "Tiaka", "Titapu", "Tiwhiri",
        "Toitiiti", "Tohu", "Tukaha", "Tumeke", "Waa", "Waikawa",
        "Weheruatanga", "Wendy", "Yasmine", "Zephyr", "Arab", "Ariki",
        "Attenborough", "Awhero", "Basil", "Blades", "Bluster",
        "Bonus", "Boss", "Clout", "Doc", "Egilsay", "Elwin", "Elliot",
        "Faulkner", "Felix", "George", "Guapo", "Gulliver", "Gumboots",
        "Henry", "Hillary", "Hokonui", "Horton", "Hugh", "Hurihuri",
        "Ian", "Jack", "Jamieson", "Jester", "Joe", "Juanma", "Kanawera",
        "Kokoto", "Komaru", "Kumi", "Luke", "Maestro", "Manu", "Matangi",
        "Merty", "Merv", "Moorhouse", "Morehu", "Moss", "Ngatapa", "Nog",
        "Oraka", "Ox", "Paddy", "Palmersan", "Percy", "Ralph", "Rangi",
        "Robbie", "Ruapuke", "Ruggedy", "Scratch", "Sinbad", "Sirocco",
        "Stumpy", "Taeatanga", "Takitimu", "Tamahou", "TauKuhurangi",
        "TeAtapo", "TeAwa", "TeHere", "TeKingi", "Tiwai", "Trevor",
        "Tuterangi", "Tutoko", "Waihopai", "Wiremu", "Wolf",
    ];

    let idx = rand::thread_rng().gen_range(0, parrot_names.len());
    parrot_names[idx]
}