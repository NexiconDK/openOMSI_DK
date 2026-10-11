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

### Vagter
- Ny indstilling **Næste tur starter ved endestationen** (slået fra fra start; findes i launcheren under Gameplay og i spillets menu). Slået til skifter vagten til næste tur, så snart du har afsluttet turen ved endestationen (holdt stille med åbne døre), i stedet for at vente til 1 minut før afgang. IBIS, destinationsskilt og navigator viser så den nye tur med det samme, og passagererne ved stoppestedet kan stige på. Slået fra virker det som før.

### Afgangstavler
- DFI-tavlerne viser nu hele teksten. Før blev den nederste halvdel af linjenumre, destinationer og minutter skåret af.
- Før midnat viser tavlerne også nattens afgange efter midnat (fx kl. 23:45: NE21 kl. 00:20).
- Tider efter midnat står nu som på et ur: **00:29** og ikke 24:29 (køreplanen tæller videre efter 24:00, men tavlen gør ikke).
- På dansk står der **Nu** i stedet for "sofort", når en bus kører nu, og det **blinker**.
- I multiplayer kommer de andre spilleres busser på tavlerne, dér hvor de faktisk er, med deres forsinkelse. Kører to spillere samme tur (fx med 1-2 minutters mellemrum), står **begge busser** på tavlen. Turens senere afgange står kun én gang. En bus, der er kørt forbi stoppestedet, forsvinder fra tavlen. Alle spillere skal have den nye version.

### Navigator (minimap og bykort)
- **Vagt-tavlen** under minimappet: turen med linjeskilt, destination og status (til tiden / forsinket / pause), stoppestederne med planlagt tid og, når du er foran eller bagud, **forventet tid** ved siden af, samt næste tur og pauser. Tavlen og vagt-panelet er taget fra Luc-nbr's openOMSI beta (MIT-licens).
- **Pilen** til højre for næste stop folder tavlen ud, så hele turens stoppesteder vises (så mange, som der er plads til), og ind igen.
- **Træk i hjørnet** nederst til højre på navigatoren for at ændre både bredde og højde. Trækker du den bred og lav, kommer vagt-tavlen **ved siden af** kortet i stedet for under det, og den viser så alle turens stop, der er plads til. Størrelsen og formen huskes.
- Vagt-tavlen viser nu kun det stop, du lige har forladt, det stop, du er på vej til, og de to næste (pilen folder stadig hele turen ud).
- Er bussen forsinket eller foran, står begge tider med navn ved hvert stop: **plan. 11:32 forv. 11:36** (den forventede i rødt/grønt), og under turens navn **planlagt 11:46 · forventet 11:50** for ankomsten til endestationen. Kører du til tiden, står kun den planlagte tid.
- Stop-knappen (stop ønsket) ligger nu til venstre for pilen og ikke oven i den, og forsinkelsen står kun én gang (på tavlen), ikke også ved næste stop.
- Et klik på minimappet åbner **bykortet** med et **vagt-panel** til venstre: alle ture med tider, pauser, linjeskift og den aktuelle tur med alle stop. Musehjulet ruller listen.
- Ny indstilling **Klassisk navigator** (slået fra fra start; i launcheren under navigatorens indstillinger og i spillets menu): navigatoren og bykortet ser ud som før, uden vagt-tavlen, vagt-panelet og H-skiltene. Kan skiftes mens du kører.
- Knappen med listen i bykortets top (grøn, når panelet vises) skjuler og viser vagt-panelet.
- Bykortet tegner nu stoppestederne som **H-skilte** med navn og tid (de kørte stop nedtonet), og de andre busser og biler i **deres rigtige størrelse og form** med linjeskilt over busserne (din egen linje fremhævet). Med vagt-panelet åbent centreres kortet på den del, panelet ikke dækker. Også fra Luc-nbr's beta.

### Turoversigt
- Ny indstilling **Turoversigt efter hver tur** (slået fra fra start; findes i launcheren under Gameplay → Passagerer og i spillets menu). Når en tur slutter, og bussen holder **helt stille** ved endestationen, åbner et vindue (aldrig under kørslen) med alle stoppesteder: planlagt og faktisk ankomst og afgang, afvigelse, **hvor mange der steg på og af**, km-tæller og status (OK, forsinket, kørt for tidligt, sprunget over). Nederst står totaler for turen.
- Påstigere og afstigere tælles nu efter, hvem der faktisk kom ind i og ud af bussen, uanset hvilken dør de brugte. Før talte kun dem, der gik ind ved føreren, så der kunne stå flere stået af end på. Dem, der stiger på ved endestationen til turen tilbage, tæller med på næste tur.
- Knapper: **Kopiér** (til udklipsholderen), **Gem som fil** (i mappen *Journeys* ved siden af kørselsloggen) og **Luk**. Esc lukker også vinduet.

### Destinationsskilt
- AI-busser, der er **ikke i rute** (Betriebsfahrt og andre destinationer, hvor alle skal af, fx Betriebshof), viser ikke længere et linjenummer på skiltet. På minimappet står der linjen og **Ikke i rute** ved bussen (fx "132 Ikke i rute").
- Linjer, hvis navn har et ord foran nummeret (fx "Linje 133"), viser nu kun nummeret på AI-bussernes skilte og på minimappet. Før stod der "LNE 133", fordi skiltets skrift ikke har små bogstaver.
- Når bussen er fyldt op til den valgte grænse (sæder og ståpladser tilsammen; **80 %, 90 %, 95 % eller 100 %**, standard 95 %), skifter destinationsskiltet hvert 3. sekund mellem linje og destination og **OPTAGET**. Skiltet tegnes af bussens eget script, så det har bussens egen skrift. Rullebånd og billeder på skiltet bliver stående. I multiplayer ser de andre spillere det også.
- Kan slås fra under **Indstillinger → Gameplay → Passagerer** (i launcheren) eller i spillets menu: "OPTAGET på destinationsskiltet, når bussen er fuld". Grænsen vælges samme sted under "OPTAGET fra". Slår du det fra, ser du det heller ikke på de andre spilleres busser.

### Lys
- Forlygterne fra en bus eller bil bag dig (andre spillere og AI) lyser ikke længere igennem din bus ind på forruden, instrumentbrættet og kabinen, når du sidder i førerhuset (grafik: Enhanced og Enhanced+).
- Det gælder nu alle lygter på andre køretøjer (også deres positionslys og de forenklede lygter), og også med grafikken Vanilla og Vanilla+.
- Din bus kaster nu skygge for lyset fra køretøjer bag dig: deres forlygter lyser ikke længere vejen, bilerne og husene **foran** din bus op, som om din bus ikke var der (det så ud som forlygter i forruden). Gælder alle grafikindstillinger, når du sidder i bussen.

### Multiplayer
- Destinationsskiltet (matrix) på de andre spilleres busser viser nu linje og destination. Før forblev skiltet ofte sort, fordi bussens script troede, at det allerede havde tegnet skiltet.
- Skiltet på de andre spilleres busser bruger nu den rigtige skrift. Før kunne destinationen stå som mærkelige tegn (eller slet ikke), fordi skrifttypernes numre blev kopieret fra den anden spillers computer, hvor de ligger i en anden rækkefølge.
- Destinationsskiltet på de andre spilleres busser tegnes nu af bussen selv på din computer ud fra deres linje og destination, som på en AI-bus. Før blev skiltets egne mellemregninger kopieret fra den anden computer, og på fx Solaris Urbino IV (Iwo) blev skiltet tomt eller viste kun prikker.
- Passagerer i de andre spilleres busser sidder nu ned på sæderne. Før stod de oprejst oven på sædet.
- Passagerer, der stiger på en anden spillers bus, beholder den destination og turlængde, de havde hos værten, i stedet for at få en tilfældig. En **gruppe** følges ad: beder bussen om én af dem, får den hele gruppen, og hos den anden spiller sidder de sammen, én køber billetter til alle, og de stiger af sammen. Begge spillere skal have den nye version.
- Ved et stoppested får en anden spillers bus nu kun de ventende, der skal med dens linje (eller ikke venter på en bestemt linje), ligesom værtens egen bus. Før fik den alle, der ventede ved stoppestedet, også dem til andre linjer, så den anden spillers bus blev fyldt, mens værtens fik langt færre.

### Passagerer
- Ny indstilling **Mennesker på gaden**: Ingen, Få, Normal, Mange eller Rigtig mange. Den bestemmer, hvor mange der går på fortovene, uafhængigt af antallet af passagerer ved stoppestederne. Findes i launcheren under Gameplay → Passagerer og i spillets menu, og virker med det samme. Skruer du ned, forsvinder de overskydende, når du ikke kan se dem.
- I din egen bus sætter passagererne sig, hvis der er en ledig plads. På din vagt står 7 ud af 10 dog op, hvis de kun skal under **1 km**. Er den ønskede slags plads optaget, tager de den anden. Ændringen kommer fra openOMSI PR #1394.
- Ny indstilling **Passagerernes lyst til at sidde**: Fra, Lidt, Normal (standard), Meget eller Rigtig meget. Findes i launcheren under Gameplay → Passagerer og i spillets menu. **Fra** giver en tilfældig ledig plads som i OMSI 2. Ellers vælger passagererne plads som rigtige mennesker:
  - De tager helst en **vinduesplads** i et **tomt dobbeltsæde** og sætter sig først ved siden af en fremmed, når der ikke er andet.
  - **Ældre passagerer** (65 år og derover efter personens fil, fx den ældre dame med gråt hår og beige cardigan) går **ofte** efter **pilotsædet**: sædet på den anden side af gangen fra føreren, lidt bag førerpladsen. Det virker også, hvis rattet sidder i højre side. De andre lader det stå, så længe de har et andet.
  - Cirka hver femte venter og rejser i en **gruppe** på 2-4 med samme destination: to tager et dobbeltsæde, tre eller fire en "firer" (to dobbeltsæder over for hinanden), ellers sidder eller står de tæt ved hinanden. Har kortet børnefigurer, kan det også være en voksen med 1-2 børn. Den første i gruppen køber **billetter til hele gruppen** hos føreren, én efter én (fx en voksenbillet og to børnebilletter), og de stiger af sammen.
  - I din bus sætter en, der står og skal et stykke endnu, sig på en plads, der bliver ledig, mens bussen kører (én ad gangen).
  - **Nu og da** (højst hvert 2½ minut) rejser en yngre passager sig for en ældre, der ikke fik en siddeplads.
  - Jo større lyst til at sidde, jo flere sætter sig på korte ture, og jo oftere flytter de, der står, hen på en ledig plads.

### Server
- På en **dedikeret server** venter der nu folk ved alle stoppesteder, efterhånden som deres afgange nærmer sig. Før kom der kun passagerer ved de stoppesteder, der havde en afgang i serverens første kvarter, og resten stod tomme hele aftenen. Ændringen kommer fra openOMSI PR #1964 (veryinsanee). Når du er vært fra dit eget spil, virkede det allerede.

### Opdateringer
- openOMSI DK opdaterer sig selv fra denne fork og ikke fra den officielle openOMSI, så ændringerne bliver.
- Der bygges kun til almindelige Windows-pc'er (x64), ikke længere til Windows på ARM (Snapdragon-bærbare).
