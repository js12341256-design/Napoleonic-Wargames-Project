/* Scenario data for "The Dusk of the Old World"
   Original Napoleonic-era grand-strategy prototype.
   Areas are laid out on a 1400x900 canvas. */

window.GAME_DATA = {
  turn: { year: 1806, month: "June", idx: 15, impulse: 3, totalImpulses: 6 },

  powers: {
    FRA: { name: "France", house: "Bonaparte", ruler: "Napoleon I", color: "#2a3a6a", pp: 142, ppDelta: +4,
           treasury: 612, income: 148, manpower: 920, flagClass: "fl-fra" },
    GBR: { name: "Great Britain", house: "Hanover", ruler: "George III", color: "#8c2a1a", pp: 128, ppDelta: +2, flagClass: "fl-gbr" },
    AUS: { name: "Austria", house: "Habsburg", ruler: "Franz II", color: "#b8a878", pp:  88, ppDelta: -3, flagClass: "fl-aus" },
    PRU: { name: "Prussia", house: "Hohenzollern", ruler: "Friedrich Wilhelm III", color: "#2b2b2b", pp: 76, ppDelta: -1, flagClass: "fl-pru" },
    RUS: { name: "Russia", house: "Romanov", ruler: "Alexander I", color: "#3a5a3a", pp: 110, ppDelta: 0, flagClass: "fl-rus" },
    SPA: { name: "Spain", house: "Bourbon", ruler: "Carlos IV", color: "#c4951f", pp:  62, ppDelta: -2, flagClass: "fl-spa" },
    OTT: { name: "Ottoman Porte", house: "Osman", ruler: "Selim III", color: "#6a3a7a", pp:  70, ppDelta: 0, flagClass: "fl-ott" },
  },

  diplomacy: {
    FRA: { GBR: "war", AUS: "friendly", PRU: "unfriendly", RUS: "war", SPA: "allied", OTT: "neutral" }
  },

  // Point-to-point areas. coords on 1400x900 viewBox.
  areas: [
    // ── France / Low Countries ──
    { id: "PARIS",      name: "Paris",        owner: "FRA", x: 410, y: 360, capital: true,  fort: 3, money: 22, mp: 18, terrain: "urban" },
    { id: "LYON",       name: "Lyon",         owner: "FRA", x: 465, y: 430, fort: 1, money: 12, mp: 9,  terrain: "open" },
    { id: "MARSEILLE",  name: "Marseille",    owner: "FRA", x: 475, y: 510, port: true, fort: 1, money: 10, mp: 7, terrain: "open" },
    { id: "BREST",      name: "Brest",        owner: "FRA", x: 320, y: 345, port: true, fort: 2, money: 7,  mp: 5, terrain: "open" },
    { id: "BOULOGNE",   name: "Boulogne",     owner: "FRA", x: 390, y: 295, port: true, fort: 1, money: 6,  mp: 6, terrain: "open" },
    { id: "ANTWERP",    name: "Antwerp",      owner: "FRA", x: 455, y: 290, port: true, fort: 2, money: 11, mp: 7, terrain: "open" },
    { id: "AMSTERDAM",  name: "Amsterdam",    owner: "FRA", x: 495, y: 255, port: true, fort: 1, money: 13, mp: 5, terrain: "marsh" },
    { id: "STRASBOURG", name: "Strasbourg",   owner: "FRA", x: 540, y: 345, fort: 2, money: 8,  mp: 7, terrain: "open" },

    // ── Iberia ──
    { id: "MADRID",     name: "Madrid",       owner: "SPA", x: 255, y: 505, capital: true, fort: 2, money: 14, mp: 11, terrain: "open" },
    { id: "BARCELONA",  name: "Barcelona",    owner: "SPA", x: 390, y: 500, port: true, fort: 1, money: 8, mp: 6, terrain: "mountain" },
    { id: "LISBON",     name: "Lisbon",       owner: "POR", x: 155, y: 525, port: true, fort: 2, money: 9, mp: 5, terrain: "open" },
    { id: "CADIZ",      name: "Cadiz",        owner: "SPA", x: 215, y: 590, port: true, fort: 1, money: 6, mp: 3, terrain: "open" },

    // ── Italy ──
    { id: "MILAN",      name: "Milan",        owner: "FRA", x: 570, y: 440, fort: 2, money: 13, mp: 9, terrain: "open" },
    { id: "VENICE",     name: "Venice",       owner: "FRA", x: 645, y: 440, port: true, fort: 2, money: 12, mp: 7, terrain: "marsh" },
    { id: "ROME",       name: "Rome",         owner: "PAP", x: 640, y: 530, fort: 1, money: 9, mp: 5, terrain: "mountain" },
    { id: "NAPLES",     name: "Naples",       owner: "NAP", x: 700, y: 590, port: true, fort: 2, money: 11, mp: 8, terrain: "mountain" },

    // ── Germany / Central ──
    { id: "MUNICH",     name: "Munich",       owner: "BAV", x: 625, y: 375, fort: 1, money: 9,  mp: 8, terrain: "open" },
    { id: "FRANKFURT",  name: "Frankfurt",    owner: "HRE", x: 575, y: 305, fort: 1, money: 10, mp: 6, terrain: "open" },
    { id: "HAMBURG",    name: "Hamburg",      owner: "DEN", x: 585, y: 225, port: true, fort: 1, money: 11, mp: 5, terrain: "open" },
    { id: "BERLIN",     name: "Berlin",       owner: "PRU", x: 685, y: 260, capital: true, fort: 3, money: 16, mp: 13, terrain: "open" },
    { id: "DRESDEN",    name: "Dresden",      owner: "SAX", x: 680, y: 320, fort: 1, money: 9, mp: 7, terrain: "open" },
    { id: "KONIGSBERG", name: "Königsberg",   owner: "PRU", x: 795, y: 240, port: true, fort: 2, money: 7, mp: 5, terrain: "open" },

    // ── Austria ──
    { id: "VIENNA",     name: "Vienna",       owner: "AUS", x: 735, y: 380, capital: true, fort: 3, money: 18, mp: 14, terrain: "open" },
    { id: "PRAGUE",     name: "Prague",       owner: "AUS", x: 685, y: 355, fort: 2, money: 11, mp: 8, terrain: "open" },
    { id: "BUDAPEST",   name: "Budapest",     owner: "AUS", x: 795, y: 410, fort: 1, money: 10, mp: 9, terrain: "open" },
    { id: "TRIESTE",    name: "Trieste",      owner: "AUS", x: 705, y: 435, port: true, fort: 1, money: 6, mp: 4, terrain: "mountain" },

    // ── Poland/East ──
    { id: "WARSAW",     name: "Warsaw",       owner: "PRU", x: 815, y: 300, fort: 1, money: 8, mp: 7, terrain: "open" },
    { id: "KRAKOW",     name: "Kraków",       owner: "AUS", x: 790, y: 350, fort: 1, money: 7, mp: 5, terrain: "open" },

    // ── Russia ──
    { id: "STPETE",     name: "St. Petersburg", owner: "RUS", x: 870, y: 165, capital: true, port: true, fort: 3, money: 15, mp: 11, terrain: "marsh" },
    { id: "MOSCOW",     name: "Moscow",       owner: "RUS", x: 1010, y: 220, capital: true, fort: 3, money: 17, mp: 16, terrain: "open" },
    { id: "RIGA",       name: "Riga",         owner: "RUS", x: 850, y: 215, port: true, fort: 2, money: 8, mp: 6, terrain: "open" },
    { id: "VILNA",      name: "Vilna",        owner: "RUS", x: 880, y: 270, fort: 1, money: 7, mp: 8, terrain: "forest" },
    { id: "SMOLENSK",   name: "Smolensk",     owner: "RUS", x: 955, y: 255, fort: 2, money: 6, mp: 9, terrain: "forest" },
    { id: "KIEV",       name: "Kiev",         owner: "RUS", x: 935, y: 355, fort: 2, money: 10, mp: 11, terrain: "open" },
    { id: "ODESSA",     name: "Odessa",       owner: "RUS", x: 920, y: 435, port: true, fort: 1, money: 7, mp: 6, terrain: "open" },

    // ── Balkans / Ottoman ──
    { id: "BUCHAREST",  name: "Bucharest",    owner: "OTT", x: 870, y: 460, fort: 1, money: 6, mp: 6, terrain: "open" },
    { id: "BELGRADE",   name: "Belgrade",     owner: "OTT", x: 795, y: 465, fort: 2, money: 7, mp: 6, terrain: "mountain" },
    { id: "SALONICA",   name: "Salonica",     owner: "OTT", x: 835, y: 540, port: true, fort: 1, money: 6, mp: 4, terrain: "mountain" },
    { id: "ISTANBUL",   name: "Istanbul",     owner: "OTT", x: 895, y: 550, capital: true, port: true, fort: 3, money: 18, mp: 13, terrain: "urban" },

    // ── British Isles ──
    { id: "LONDON",     name: "London",       owner: "GBR", x: 370, y: 230, capital: true, port: true, fort: 3, money: 28, mp: 14, terrain: "urban" },
    { id: "EDINBURGH",  name: "Edinburgh",    owner: "GBR", x: 330, y: 160, port: true, fort: 2, money: 8, mp: 5, terrain: "open" },
    { id: "DUBLIN",     name: "Dublin",       owner: "GBR", x: 245, y: 200, port: true, fort: 1, money: 6, mp: 4, terrain: "open" },

    // ── Scandinavia ──
    { id: "COPENHAGEN", name: "Copenhagen",   owner: "DEN", x: 615, y: 185, port: true, fort: 2, money: 9, mp: 4, terrain: "open" },
    { id: "STOCKHOLM",  name: "Stockholm",    owner: "SWE", x: 680, y: 135, port: true, fort: 2, money: 8, mp: 6, terrain: "forest" },
  ],

  // Connections (edges)
  edges: [
    ["PARIS","LYON"],["PARIS","BOULOGNE"],["PARIS","BREST"],["PARIS","STRASBOURG"],["PARIS","ANTWERP"],
    ["LYON","MARSEILLE"],["LYON","STRASBOURG"],["LYON","MILAN"],["LYON","BARCELONA"],
    ["BOULOGNE","ANTWERP"],["ANTWERP","AMSTERDAM"],["AMSTERDAM","HAMBURG"],["ANTWERP","FRANKFURT"],
    ["STRASBOURG","FRANKFURT"],["STRASBOURG","MUNICH"],["FRANKFURT","MUNICH"],["FRANKFURT","DRESDEN"],
    ["FRANKFURT","HAMBURG"],["HAMBURG","COPENHAGEN"],["HAMBURG","BERLIN"],["COPENHAGEN","STOCKHOLM"],
    ["BERLIN","DRESDEN"],["BERLIN","WARSAW"],["BERLIN","KONIGSBERG"],["DRESDEN","PRAGUE"],
    ["PRAGUE","VIENNA"],["VIENNA","MUNICH"],["VIENNA","BUDAPEST"],["VIENNA","TRIESTE"],["VIENNA","KRAKOW"],
    ["MUNICH","MILAN"],["MILAN","VENICE"],["VENICE","TRIESTE"],["VENICE","ROME"],["ROME","NAPLES"],
    ["BUDAPEST","KRAKOW"],["BUDAPEST","BELGRADE"],["KRAKOW","WARSAW"],["WARSAW","KONIGSBERG"],["WARSAW","VILNA"],
    ["KONIGSBERG","RIGA"],["RIGA","STPETE"],["RIGA","VILNA"],["VILNA","SMOLENSK"],["VILNA","KIEV"],
    ["SMOLENSK","MOSCOW"],["SMOLENSK","STPETE"],["KIEV","ODESSA"],["KIEV","BUCHAREST"],
    ["ODESSA","BUCHAREST"],["BUCHAREST","BELGRADE"],["BELGRADE","SALONICA"],["SALONICA","ISTANBUL"],
    ["BUCHAREST","ISTANBUL"],
    ["MADRID","BARCELONA"],["MADRID","LISBON"],["MADRID","CADIZ"],
    ["LONDON","EDINBURGH"],["LONDON","DUBLIN"],
    ["PARIS","MADRID"], // via Pyrenees pass abstraction
    ["TRIESTE","BELGRADE"],["ROME","TRIESTE"],
  ],

  // Sea links (dotted)
  seaLinks: [
    ["BOULOGNE","LONDON"],["BREST","LONDON"],["AMSTERDAM","LONDON"],
    ["LISBON","LONDON"],["CADIZ","LONDON"],
    ["LONDON","EDINBURGH"],["LONDON","DUBLIN"],
    ["COPENHAGEN","STOCKHOLM"],["STOCKHOLM","STPETE"],
    ["MARSEILLE","BARCELONA"],["BARCELONA","NAPLES"],["NAPLES","SALONICA"],
    ["ISTANBUL","ODESSA"],["ODESSA","SALONICA"],
    ["TRIESTE","VENICE"],
  ],

  corps: [
    // French corps (player)
    { id: "IGARDE",   name: "I. Garde Impériale", owner: "FRA", area: "STRASBOURG", inf: 22, cav: 4, art: 6, morale: 0.95, supply: true,  leader: "NAPOLEON", hidden: false, moved: 0 },
    { id: "IIIDAVOUT",name: "III. Corps",         owner: "FRA", area: "FRANKFURT",  inf: 16, cav: 3, art: 4, morale: 0.9,  supply: true,  leader: "DAVOUT",   hidden: false, moved: 2 },
    { id: "IVSOULT",  name: "IV. Corps",          owner: "FRA", area: "MUNICH",     inf: 14, cav: 2, art: 3, morale: 0.85, supply: true,  leader: "SOULT",    hidden: false, moved: 0 },
    { id: "VIINEY",   name: "VI. Corps",          owner: "FRA", area: "MILAN",      inf: 12, cav: 2, art: 3, morale: 0.8,  supply: true,  leader: "NEY",      hidden: false, moved: 0 },
    { id: "IIBERNAD", name: "II. Corps",          owner: "FRA", area: "ANTWERP",    inf: 13, cav: 2, art: 2, morale: 0.9,  supply: true,  leader: "BERNADOTTE",hidden: false, moved: 0 },
    { id: "VMASSENA", name: "V. Corps",           owner: "FRA", area: "VENICE",     inf: 11, cav: 1, art: 3, morale: 0.75, supply: false, leader: "MASSENA",  hidden: false, moved: 0 },
    { id: "RESSPAIN", name: "Armée d'Espagne",    owner: "FRA", area: "MADRID",     inf: 8,  cav: 2, art: 1, morale: 0.7,  supply: true,  leader: null,       hidden: true,  moved: 0 },

    // Enemies visible
    { id: "RUSBAGRA", name: "1st Western Army",   owner: "RUS", area: "VILNA",      inf: 18, cav: 4, art: 4, morale: 0.85, supply: true, leader: "BAGRATION", hidden: true, moved: 0 },
    { id: "RUSKUTUZ", name: "2nd Western Army",   owner: "RUS", area: "SMOLENSK",   inf: 20, cav: 3, art: 5, morale: 0.8,  supply: true, leader: "KUTUZOV",   hidden: true, moved: 0 },
    { id: "PRUBLUCH", name: "Prussian Main",      owner: "PRU", area: "BERLIN",     inf: 16, cav: 3, art: 4, morale: 0.85, supply: true, leader: "BLÜCHER",   hidden: true, moved: 0 },
    { id: "AUSCHARLES",name:"Erzherzog Karl",     owner: "AUS", area: "VIENNA",     inf: 17, cav: 3, art: 4, morale: 0.8,  supply: true, leader: "CHARLES",   hidden: true, moved: 0 },
    { id: "GBRWELL",  name: "Expeditionary",      owner: "GBR", area: "LISBON",     inf: 10, cav: 1, art: 2, morale: 0.9,  supply: true, leader: "WELLESLEY", hidden: true, moved: 0 },
  ],

  leaders: {
    NAPOLEON:   { name: "Napoleon I",   strat: 5, tac: 5, init: 9, army: true },
    DAVOUT:     { name: "Davout",       strat: 4, tac: 5, init: 7, army: true },
    SOULT:      { name: "Soult",        strat: 3, tac: 4, init: 6 },
    NEY:        { name: "Ney",          strat: 2, tac: 5, init: 7 },
    BERNADOTTE: { name: "Bernadotte",   strat: 3, tac: 3, init: 5 },
    MASSENA:    { name: "Masséna",      strat: 4, tac: 4, init: 6 },
    BAGRATION:  { name: "Bagration",    strat: 3, tac: 4, init: 6 },
    KUTUZOV:    { name: "Kutuzov",      strat: 4, tac: 3, init: 5 },
    BLÜCHER:    { name: "Blücher",      strat: 3, tac: 3, init: 5 },
    CHARLES:    { name: "Archduke Charles", strat: 4, tac: 3, init: 5 },
    WELLESLEY:  { name: "Wellesley",    strat: 4, tac: 4, init: 6 },
  },

  // Impulse queue for current turn
  impulseQueue: [
    { impulse: 1, power: "FRA", roll: 9, status: "done",   note: "Advanced III. Corps → Frankfurt" },
    { impulse: 2, power: "RUS", roll: 7, status: "done",   note: "Bagration screened westward" },
    { impulse: 3, power: "FRA", roll: 6, status: "active", note: "— awaiting orders —" },
    { impulse: 4, power: "AUS", roll: 5, status: "pending",note: "" },
    { impulse: 5, power: "PRU", roll: 4, status: "pending",note: "" },
    { impulse: 6, power: "GBR", roll: 3, status: "pending",note: "" },
  ],

  dispatches: [
    { from: "Cabinet of Berlin", subject: "Mobilisation confirmed", body: "Prussian reserves called to Saxony; Blücher rides south at dawn.", seal: true },
    { from: "Ambassador, Vienna", subject: "Overtures from Metternich", body: "Austria proposes mutual access through Bohemia — pending neutrality guarantees.", seal: false },
    { from: "Quartermaster, Venice", subject: "Magazines exhausted", body: "V. Corps requires reprovisioning within the fortnight or must forage.", seal: true },
  ],

  turnLog: [
    { t: "09:14", c: "econ",    e: "Income posted: ₣148 (farms +92, tariff +34, colonial +22)" },
    { t: "09:14", c: "econ",    e: "Conscription drew 28 SP manpower from depots Est & Bavière." },
    { t: "09:32", c: "diplo",   e: "Vienna raised its status toward Paris to Friendly." },
    { t: "09:32", c: "diplo",   e: "Madrid affirmed alliance; subsidy of ₣12 tendered." },
    { t: "10:05", c: "diplo",   e: "Berlin lodged an unfriendly circular at the Diet." },
    { t: "10:18", c: "normal",  e: "Impulse 1 — France rolled 9; initiative secured." },
    { t: "10:18", c: "normal",  e: "III. Corps marched Antwerp → Frankfurt (4 MP)." },
    { t: "10:42", c: "normal",  e: "Impulse 2 — Russia rolled 7; screens thrown forward." },
    { t: "10:42", c: "highlight",e: "Bagration's outriders sighted on the Niemen." },
    { t: "11:01", c: "combat",  e: "Skirmish at Vilna — Russian cavalry withdrew, losses negligible." },
    { t: "11:15", c: "normal",  e: "Impulse 3 opens. France to move." },
  ],

  ppHistory: {
    FRA: [95,98,102,108,115,120,124,128,130,134,138,140,142],
    GBR: [130,132,131,130,129,128,127,128,128,129,128,128,128],
    AUS: [105,104,100, 98, 95, 94, 92, 91, 90, 90, 89, 90, 88],
    PRU: [ 85, 84, 82, 81, 80, 79, 78, 78, 77, 77, 77, 76, 76],
    RUS: [110,110,111,110,110,109,110,110,110,110,110,110,110],
    SPA: [ 72, 70, 69, 67, 66, 65, 64, 63, 63, 63, 62, 62, 62],
    OTT: [ 70, 70, 70, 70, 71, 71, 71, 70, 70, 70, 70, 70, 70],
  },
};
