## openOMSI DK

Vores egen udgave af openOMSI, bygget på den officielle openOMSI med disse ændringer.

### Dansk
- Spillet og launcheren findes nu på dansk: vælg **Dansk** under sprog i indstillingerne. Alle 1.610 tekster er oversat.

### Natbusser og vagter over midnat
- Ture med tider efter midnat (fx **24:10** og **25:10**) kører nu natten efter vagtens egen dag. En fredagsvagts tur kl. 24:10 kører lørdag kl. 00:10, og lørdagsvagtens først natten til søndag.
- AI-bussen fortsætter sin vagt hen over midnat og forsvinder ikke.
- Listen fredag aften viser ikke længere torsdagens og lørdagens natvagter.
- Info-vinduet for en vagt åbner på den tur, der kører nu eller kører næste gang, også efter midnat. Kl. 02:18 vælger det fx turen kl. 02:50 og ikke aftenens første tur. Pilene viser stadig alle ture.

### Lister over linjer og vagter
- En vagt vises fra **2 timer før** den starter og indtil **1 time efter** den er slut, både i spillet og i launcheren.
- Launcheren viser kun dagens vagter.
- Efter midnat vises kun gårsdagens natvagter, der stadig kører. Lørdagens natvagt (fx fra kl. 22:44) vises derfor ikke lørdag kl. 02:47, men først fra kl. 20:44.

### Afgangstavler
- DFI-tavlerne viser nu hele teksten. Før blev den nederste halvdel af linjenumre, destinationer og minutter skåret af.
- Før midnat viser tavlerne også nattens afgange efter midnat (fx kl. 23:45: NE21 kl. 00:20).

### Turoversigt
- Ny indstilling **Turoversigt efter hver tur** (slået fra fra start; findes i launcheren under Gameplay → Passagerer og i spillets menu). Når en tur slutter, åbner et vindue med alle stoppesteder: planlagt og faktisk ankomst og afgang, afvigelse, **hvor mange der steg på og af**, km-tæller og status (OK, forsinket, kørt for tidligt, sprunget over). Nederst står totaler for turen.
- Knapper: **Kopiér** (til udklipsholderen), **Gem som fil** (i mappen *Journeys* ved siden af kørselsloggen) og **Luk**. Esc lukker også vinduet.

### Destinationsskilt
- AI-busser, der er **ikke i rute** (Betriebsfahrt og andre destinationer, hvor alle skal af, fx Betriebshof), viser ikke længere et linjenummer på skiltet. På minimappet står der linjen og **Ikke i rute** ved bussen (fx "132 Ikke i rute").
- Linjer, hvis navn har et ord foran nummeret (fx "Linje 133"), viser nu kun nummeret på AI-bussernes skilte og på minimappet. Før stod der "LNE 133", fordi skiltets skrift ikke har små bogstaver.
- Når bussen er fyldt op til den valgte grænse (sæder og ståpladser tilsammen; **80 %, 90 %, 95 % eller 100 %**, standard 95 %), skifter destinationsskiltet hvert 3. sekund mellem linje og destination og **OPTAGET**. Skiltet tegnes af bussens eget script, så det har bussens egen skrift. Rullebånd og billeder på skiltet bliver stående. I multiplayer ser de andre spillere det også.
- Kan slås fra under **Indstillinger → Gameplay → Passagerer** (i launcheren) eller i spillets menu: "OPTAGET på destinationsskiltet, når bussen er fuld". Grænsen vælges samme sted under "OPTAGET fra". Slår du det fra, ser du det heller ikke på de andre spilleres busser.

### Lys
- Forlygterne fra en bus eller bil bag dig (andre spillere og AI) lyser ikke længere igennem din bus ind på forruden, instrumentbrættet og kabinen, når du sidder i førerhuset (grafik: Enhanced og Enhanced+).

### Multiplayer
- Destinationsskiltet (matrix) på de andre spilleres busser viser nu linje og destination. Før forblev skiltet ofte sort, fordi bussens script troede, at det allerede havde tegnet skiltet.
- Passagerer i de andre spilleres busser sidder nu ned på sæderne. Før stod de oprejst oven på sædet.

### Passagerer
- Ny indstilling **Mennesker på gaden**: Ingen, Få, Normal, Mange eller Rigtig mange. Den bestemmer, hvor mange der går på fortovene, uafhængigt af antallet af passagerer ved stoppestederne. Findes i launcheren under Gameplay → Passagerer og i spillets menu, og virker med det samme. Skruer du ned, forsvinder de overskydende, når du ikke kan se dem.
- I din egen bus sætter passagererne sig, hvis der er en ledig plads. På din vagt står 7 ud af 10 dog op, hvis de kun skal under **1 km**. Er den ønskede slags plads optaget, tager de den anden. Ændringen kommer fra openOMSI PR #1394.
- AI-bussernes passagerer tager stadig en tilfældig plads, som i OMSI 2.

### Opdateringer
- openOMSI DK opdaterer sig selv fra denne fork og ikke fra den officielle openOMSI, så ændringerne bliver.
