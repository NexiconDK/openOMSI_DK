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
- Hurtigere: kun tavler inden for ca. 120 m blinker. Tavler længere væk viser "Nu" uden at blinke. Før blev teksten på alle tavler i byen tegnet forfra to gange i sekundet, hvilket kunne give små hak.
- I multiplayer kommer de andre spilleres busser på tavlerne, dér hvor de faktisk er, med deres forsinkelse. Kører to spillere samme tur (fx med 1-2 minutters mellemrum), står **begge busser** på tavlen. Turens senere afgange står kun én gang. En bus, der er kørt forbi stoppestedet, forsvinder fra tavlen. Alle spillere skal have den nye version.

### Navigator (minimap og bykort)
- **Vagt-tavlen** under minimappet: turen med linjeskilt, destination og status (til tiden / forsinket / pause), stoppestederne med planlagt tid og, når du er foran eller bagud, **forventet tid** ved siden af, samt næste tur og pauser. Tavlen og vagt-panelet er taget fra Luc-nbr's openOMSI beta (MIT-licens).
- **Pilen** til højre for næste stop folder tavlen ud, så hele turens stoppesteder vises (så mange, som der er plads til), og ind igen.
- **Træk i hjørnet** nederst til højre på navigatoren for at ændre både bredde og højde. Trækker du den bred og lav, kommer vagt-tavlen **ved siden af** kortet i stedet for under det, og den viser så alle turens stop, der er plads til. Størrelsen og formen huskes.
- Vagt-tavlen viser nu kun det stop, du lige har forladt, det stop, du er på vej til, og de to næste (pilen folder stadig hele turen ud).
- Status-boksen på vagt-tavlen viser nu **præcis hvor meget** du er foran eller bagud, i minutter og sekunder: **rød 00:32**, når du er foran køreplanen (bare ét sekund er nok), **grøn Til tiden - 01:45** fra 0 til 2 minutter bagud, og **orange 02:15**, når du er mere end 2 minutter forsinket. Timer står kun med, når det er over en time (fx 01:02:05). Før stod der bare "til tiden" helt op til 3 minutter, og foran var blå.
- Er bussen forsinket eller foran, står begge tider med navn ved hvert stop: **plan. 11:32 forv. 11:36** (den forventede i samme farve som status-boksen), og under turens navn **planlagt 11:46 · forventet 11:50** for ankomsten til endestationen. Kører du til tiden, står kun den planlagte tid.
- Stop-knappen (stop ønsket) ligger nu til venstre for pilen og ikke oven i den, og forsinkelsen står kun én gang (på tavlen), ikke også ved næste stop.
- Ny indstilling **Biler på kortet** (slået til fra start; i launcheren under navigatorens indstillinger og i spillets menu). Slået fra viser minimappet og bykortet kun busser, trolleybusser og sporvogne, ikke de blå prikker for bilerne. Virker, når **AI-køretøjer på kortet** er slået til, og kan skiftes mens du kører.
- Et klik på minimappet åbner **bykortet** med et **vagt-panel** til venstre: alle ture med tider, pauser, linjeskift og den aktuelle tur med alle stop. Musehjulet ruller listen.
- Ny indstilling **Klassisk navigator** (slået fra fra start; i launcheren under navigatorens indstillinger og i spillets menu): navigatoren og bykortet ser ud som før, uden vagt-tavlen, vagt-panelet og H-skiltene. Kan skiftes mens du kører.
- Vagt-panelet på bykortet viser nu **vagtens navn** i overskriften, fx **Vagt 143H01** (navnet fra køreplanen).
- Knappen med listen i bykortets top (grøn, når panelet vises) skjuler og viser vagt-panelet.
- Bykortet tegner nu stoppestederne som **H-skilte** med navn og tid (de kørte stop nedtonet), og de andre busser og biler i **deres rigtige størrelse og form** med linjeskilt over busserne (din egen linje fremhævet). Med vagt-panelet åbent centreres kortet på den del, panelet ikke dækker. Også fra Luc-nbr's beta.
- Hurtigere: H-skiltene på bykortet tegnes én gang i hver størrelse og genbruges, i stedet for at blive bygget forfra hvert billede.

### Web-IBIS (tablet)
- Ny indstilling **Web-IBIS på en tablet eller telefon** (slået fra fra start; i launcheren under navigatorens indstillinger og i spillets menu). Slået til viser spillet en IBIS som webside på dit eget netværk. Adressen står øverst til venstre i spillet, fx **Web-IBIS: http://192.168.1.20:8095**. Åbn den i browseren på en tablet eller telefon på samme wifi. Første gang spørger Windows' firewall, om spillet må bruge netværket. Svar ja for private netværk.
- **Login:** skriv et vilkårligt chaufførnummer og en vilkårlig adgangskode. Klokken og datoen i hjørnet følger spillets tid.
- **Linjer:** alle linjer i kortets køreplan, som bussens HOF-fil kender. Den linje, du kører, er markeret.
- Rettet: linjelisten var tom ("Ingen køreplan på dette kort"), fordi køreplanen kalder linjerne "Linje 143", mens HOF-filen kender dem som "143". Nu genkendes begge, og kender HOF-filen ingen af linjerne, vises de alle.
- **Ture:** et tryk på en linje viser dagens ture i begge retninger med afgangstid, fra/til og vagt. Kørte ture er grå, og listen starter ved den næste tur. Knapperne øverst filtrerer på retning.
- **Kør tur:** tryk på en tur og svar **Ja**. Spillet starter turen som din vagt (bussen bliver, hvor den er), og bussen skilter destinationen.
- **Vagt:** linje og destination, status-boksen, forsinkelse, tur x/y, næste stop og hele vagten med turens stop og forventede tider.
- **Dine egne billeder:** læg login-billedet som **login.png** (1280 × 720) i mappen **WebIBIS** i din OMSI-mappe. Billedets øverste stribe bruges også som topbjælke på de andre skærme. Uden billede tegnes simple hvide skærme. Billederne følger ikke med spillet.

### Turoversigt
- Ny indstilling **Turoversigt efter hver tur** (slået fra fra start; findes i launcheren under Gameplay → Passagerer og i spillets menu). Når en tur slutter, og bussen holder **helt stille** ved endestationen, åbner et vindue (aldrig under kørslen) med alle stoppesteder: planlagt og faktisk ankomst og afgang, afvigelse, **hvor mange der steg på og af**, km-tæller og status (OK, forsinket, kørt for tidligt, sprunget over). Nederst står totaler for turen.
- Påstigere og afstigere tælles nu efter, hvem der faktisk kom ind i og ud af bussen, uanset hvilken dør de brugte. Før talte kun dem, der gik ind ved føreren, så der kunne stå flere stået af end på. Dem, der stiger på ved endestationen til turen tilbage, tæller med på næste tur.
- Knapper: **Kopiér** (til udklipsholderen), **Gem som fil** (i mappen *Journeys* ved siden af kørselsloggen) og **Luk**. Esc lukker også vinduet.

### Destinationsskilt
- AI-busser, der er **ikke i rute** (Betriebsfahrt og andre destinationer, hvor alle skal af, fx Betriebshof), viser ikke længere et linjenummer på skiltet. På minimappet står der linjen og **Ikke i rute** ved bussen (fx "132 Ikke i rute").
- Linjer, hvis navn har et ord foran nummeret (fx "Linje 133"), viser nu kun nummeret på AI-bussernes skilte og på minimappet. Før stod der "LNE 133", fordi skiltets skrift ikke har små bogstaver.
- Når bussen er fyldt op til den valgte grænse (sæder og ståpladser tilsammen; **80 %, 90 %, 95 % eller 100 %**, standard 95 %), skifter destinationsskiltet hvert 3. sekund mellem linje og destination og **OPTAGET**. Skiltet tegnes af bussens eget script, så det har bussens egen skrift. Rullebånd og billeder på skiltet bliver stående. I multiplayer ser de andre spillere det også.
- Kan slås fra under **Indstillinger → Gameplay → Passagerer** (i launcheren) eller i spillets menu: "OPTAGET på destinationsskiltet, når bussen er fuld". Grænsen vælges samme sted under "OPTAGET fra". Slår du det fra, ser du det heller ikke på de andre spilleres busser.

### Snestorm
Sat på en kontakt, så almindelig sne er præcis som før. I din .cmd før `openomsi.exe`:

```
set OMSI_SNOWSTORM=1
```

Tallet går fra 0 til 1, og **dialen er jævn** — hver tiendedel lukker udsynet med samme andel, så 0,5 er halvvejs og ikke "næsten ingenting". Det sner mindst så hårdt, uanset hvad vejrfilen siger, så du kan tænde den og se det med det samme.

Hvad der sker ved fuld styrke:

- **Sigtbarheden går fra 300 m til omkring 40 m.** Et snefog, hvor du ser kølerhjelmen og ikke meget mere. (Definitionen på en snestorm er under 400 m, og et whiteout er nede i titusindvis af meter.)
- **Dobbelt så mange fnug** — 300.000 i stedet for 150.000 — og større klumper, for i kraftig sne fryser krystallerne sammen.
- **Kuling med vindstød.** Vinden hæves til 17 m/s i vejrets egen retning, hvis vejrfilen siger mindre, og oven på den ligger to langsomme bølger, der aldrig helt rammer sammen, så det kommer i pust og lun i stedet for at blæse fladt. Stødene er regnet som den præcise integral af vindhastigheden, så et fnug aldrig springer, når vinden drejer.
- **Fnuggene trækker streger.** Ved 17 m/s flyver et fnug næsten vandret og krydser en god del af en meter, mens øjet holder ét billede — en rund prik ser derfor ud som stillestående luft med sne hængende i. Hvert fnug tegnes nu langs sin egen bane (vind, vindstød, sving og fald), og det, det dækker, fordeles over længden, så en lang streg er en svag streg.

`OMSI_SNOWSTORM=0` (standard) ændrer ingenting — vejrets egen sne er bit for bit som før.

### Sne på vejene, hjulspor og dækspor
Fra openOMSI PR #1902 af **Bulekoo** (MIT-licens, med kredit).

Før var vejrets "sne på vejen" en kontakt: vejbanen var hvid eller bar fra det ene øjeblik til det andet, og kortets `WinterSnowfall`-billeder fulgte med. Nu har vejene deres eget snelag:

- **Sneen lægger sig, mens det sner.** Den starter, som vejret har den, og vokser, mens det sner: et kraftigt snefald (Starker Schneefall) lukker en vej i cirka **10 minutter**. Den smelter igen, når luften er over 0,5 °C, og hurtigere i regn.
- **Den lægger sig i pletter** først — en let fnugning i pletter på en meter eller to og i asfaltens korn — og lukker sig, efterhånden som der falder mere.
- **Hjulspor:** trafikken holder to striber i hver vejbane nede i **våd sjap**, som er mørkere og blankere end sneen ved siden af. Sporene ligger ikke på en skinne: de vandrer en håndsbredde, bliver bredere og smallere langs vejen og er slidt dybere nogle steder. En travl gade har friske spor, en stille vej (en busbane, en endestationssløjfe) er halvt fyldt igen.
- **Dækspor:** hvert køretøj inden for **110 m** af kameraet presser sine dæk ned i sneen — din egen bus og alle AI-biler. Sporet er fladt og gråt med mønsterets korn, og har bløde kanter.
- **Sporene fyldes igen** af den sne, der falder, efter at hjulet kørte: et dæksporn begynder at sløres efter 8 % af et fuldt lag og er væk ved 28 %, et hjulspor fra 15 % til 52 %. I kraftig sne er det et par minutter for et dækspor og fem-syv for et hjulspor — holder du stille ved et stoppested længe nok, kan du se det ske.
- **Sne på tagene:** hvert køretøj nær kameraet samler sne på taget. Den lægger sig i pletter først og bliver et lukket lag. Kører du hurtigt, **blæser luftstrømmen den af**: cirka 400 m ved 50 km/h, 200 m ved 80. Den smelter sammen med vejens sne. Mængden rider med i køretøjets eget mesh, så pletterne følger bussen i stedet for at ligge stille i verden.

Feltet med spor er en **256 m firkant** omkring kameraet (12,5 cm pr. punkt), der går rundt om verden, så kun de felter, kameraet kører ind i, skrives om. Hjulsporene tegnes forfra, når vejnettet er holdt op med at vokse (nye felter kommer med deres vejbaner nogle få ad gangen).

Hvad der ikke sker:

- Vælger du grafikken **Vanilla (klassisk)**, sker der ingenting: der viser spillet sne som OMSI 2 gennem kortets egne `WinterSnow`-billeder. Med sneen slået til tages kortets `WinterSnowfall`-vejbilleder ikke.
- På **OpenGL** (og på kort, hvor der ikke er en tekstur-plads tilbage) ligger sporfeltet ikke på kortet, og vejene er som før.

Indstillinger, i din .cmd før `openomsi.exe`:

```
set OMSI_ROAD_SNOW=off      ; tilbage til vejrets gamle kontakt
set OMSI_ROAD_SNOW=0.5      ; start med et halvt lag på vejene
set OMSI_ROAD_SNOW_GRID=1   ; tegn et 4 m gitter i feltet i stedet for hjulsporene
set OMSI_ROAD_SNOW_LOG=1    ; skriv lagets tykkelse og vejret i loggen hvert 10. sekund
```

Standard er `on`: sneen begynder, som vejrfilen siger, og vokser og smelter derfra.

### Lys
- Forlygterne fra en bus eller bil bag dig (andre spillere og AI) lyser ikke længere igennem din bus ind på forruden, instrumentbrættet og kabinen, når du sidder i førerhuset (grafik: Enhanced og Enhanced+).
- Det gælder nu alle lygter på andre køretøjer (også deres positionslys og de forenklede lygter), og også med grafikken Vanilla og Vanilla+.
- Din bus kaster nu skygge for lyset fra køretøjer bag dig: deres forlygter lyser ikke længere vejen, bilerne og husene **foran** din bus op, som om din bus ikke var der (det så ud som forlygter i forruden). Gælder alle grafikindstillinger, når du sidder i bussen.
- Hurtigere: skyggen regnes nu kun ud for lygter, hvis lys kan nå forbi din bus. Før blev hvert billedpunkt tjekket mod alle bilers lygter i byen, hvilket kostede om natten.

### Ydelse (testbuild)
Første runde rettelser fra gennemgangen af hele koden. Du skal ikke ændre nogen indstilling: det virker af sig selv.

**Om natten**
- **Lysgitteret:** hver lygte bliver nu skrevet ind i de felter, dens **cirkel** dækker, i stedet for i hele den firkant, dens rækkevidde dækkede. Hvert felt har plads til 32 lygter, og når det er fuldt, giver den svageste lygte nu plads til en stærkere, i stedet for at den sidst ankomne altid blev smidt ud. (Et forsøg på også at bruge lygtens **kegle** blev rullet tilbage: tågen, disen og den faldende sne slår lygten op i det felt, hvor synslinjen er tættest på den, og det punkt kan ligge i en hvilken som helst retning fra lygten. Med keglen manglede lygten i de felter, så det oplyste tågeskær endte brat ved feltgrænserne som kasser foran bussen.)
- **Køretøjslygter:** lygterne på en bil længere væk end 1100 m bliver ikke længere regnet ud. De kunne alligevel ikke nå ind i lysgitteret omkring kameraet. Baglygter og andre glimt ses stadig på al afstand.
- **Forlygternes placering** på bussen (hvor de sidder, og hvor langt de to kegler er fra hinanden) regnes nu ud én gang pr. bustype i stedet for for hver bil i hvert billede.
- **Spejle:** et spejl viser et lille billede, og det tegnes nu kun 500 m ud og uden de mindste detaljer (fire gange større krav end på skærmen). Før blev hvert spejl stillet op med samme detaljegrad og samme sigtbarhed som hele skærmen.
- **Lyslisten deles** nu mellem skærmen og spejlene, som koden hele tiden har sagt, den gjorde. Fordi spejle og skærm regnede hver sin liste, blev hele lysgitteret (512 KB) sendt til grafikkortet igen for hver visning, hvert billede.

**Hak og frysninger**
- **Skift af opløsning** bygger ikke længere scenen op fra bunden. Før blev alle objekters data lagt i nye buffere, alle afgrænsninger regnet om, og alle mellembilleder smidt væk, hver gang billedstørrelsen ændrede sig. Det sker, når den automatiske opløsning skruer ned, altså præcis når FPS falder: trinnet, der skulle give FPS tilbage, kostede selv et hak.
- **Lyden venter ikke længere på sig selv.** Spillet spurgte op mod tusind gange pr. billede, om en lyd stadig spiller, og hvert spørgsmål kunne vente på, at lydtråden blev færdig med at blande en hel lydblok. Nu svares der med det samme fra en liste for sig.
- **Sporing af lydhændelser:** de voksede uden grænse for biler længere væk end 300 m, fordi ingen hentede dem. En af listerne gemmer en kopi af alle scriptvariabler pr. hændelse. Nu ryddes de altid, og de kan højst indeholde 64. Det fjerner også det "bump", der kom, når en bil kom tilbage i nærheden og afspillede hele puklen i ét billede.

**Grafikkortets hukommelse**
- **`texture_memory` kan ikke længere sættes højere, end kortet kan holde ved siden af billederne.** Et kort rummer mere end teksturerne: maskerne, de billeder der tegnes i (ved 4x MSAA fire gange hver), skyggekortene, spejlene og driverens eget. Indstillingen må nu gå en femtedel over det, programmet selv har regnet kortet til, men aldrig over to femtedele af kortet.
  Før blev en sat værdi lukket op til halvdelen af kortet og en femtedel oveni: **3751 MB på et 6 GB-kort**, hvor programmet selv havde sat netop det kort til 1600 MB. To spillere havde `texture_memory 6000` i deres indstillinger og et RTX 3050: teksturerne stod på 3300 MB, kortet kunne ikke holde det ved siden af resten, og driveren flyttede teksturer over bussen for hvert billede. Resultatet var **9 FPS med 80 % af hvert billede brugt på at vente på at vise det** — og det hjalp ingenting at skrue opløsningen ned til 55 %, slå skygger og ambient occlusion fra, for det var ikke der, kortet manglede noget. Nu er loftet 2400 MB på et 6 GB-kort og 4804 MB på et 12 GB-kort. Fire tests.
  `OMSI_TEXTURE_MEMORY=<MB>` sætter stadig en anden værdi, hvis man vil prøve.

**Hele tiden**
- **Stille lyde** bliver ikke længere regnet om prøve for prøve. En fjern bus holder sine motorlyde i gang på -30 til -40 dB, og de blev blandet alligevel. De følger stadig tiden, så en løkke kommer tilbage i samme fase.
- **Scriptopslag:** "skriver scriptet til denne variabel?" blev afgjort ved at gennemgå hvert eneste trin i hele bussens script, flere gange pr. billede pr. bus ved et stoppested. Nu slås det op i en liste, der laves én gang.
- **Skærmens opdateringsfrekvens** blev slået op i skærmdriveren i hvert billede. Nu højst én gang i sekundet.
- **LAN:** din bus' data bliver nu kun samlet sammen de gange, de faktisk skal sendes (10 gange i sekundet). Ved 144 FPS blev 130 af 144 omgange samlet og smidt væk. Og listen over, hvilke variabler de andre spilleres skilte og rullegardiner skrives gennem, laves nu én gang pr. bus i stedet for i hvert billede (den gennemgik hele modellens masker og materialer og sorterede dem).

### Lygternes skær i tåge, dis og sne
- **Kasserne i lyset er væk.** I tåge, dis og snevejr tegnede spillet forlygternes skær i luften som hårde, firkantede områder foran bussen, med et groft prikmønster indeni. Fejlen har været der hele tiden, også i openOMSI (som forsøgte at rette den to gange i lyset på vejen, 58d963ac og 2da18118, og rullede begge tilbage med ordene om, at det firkantede sad i skæret, ikke i vejlyset, eb830952).
  Skæret regnes ud ved at tage 16 prøver langs synslinjen. Prøverne blev fordelt efter afstanden fra synslinjen til lygten, hvilket er helt rigtigt for en gadelygte, der lyser lige meget til alle sider. For en **forlygte** er det omvendt: den sender intet ud de første meter og lyser vejen fra ti meter og ud. Sidder man i førerhuset, er der kun halvanden meter til lygten, og så landede 15 af de 16 prøver inde i de første par meter, hvor der ikke er noget lys, mens hele den oplyste strækning skulle klares af den ene prøve, der var tilbage. Hver billedpunkt fik derfor et tilfældigt resultat mellem ingenting og næsten det dobbelte. Det er prikmønsteret. Og da den ene prøves placering afhænger af, hvor langt der er til det, synslinjen rammer, sprang skæret, hver gang afstanden sprang, altså langs kanterne af vejen, fortovet og bilerne. Det er kasserne.
  Nu fordeles prøverne over den strækning, lygten faktisk lyser op, og hver prøve bærer sin egen længde og sin egen afstand til lygten præcist. En lygte, synslinjen passerer tæt forbi, får stadig prøverne tættest på sig, så gadelygternes bløde skær er uændret. Udregnet på forhånd falder udsvinget mellem to nabobilledpunkter fra "mellem 0 og 178 % af det rigtige" til omkring ±10 %.
- **Skæret er skruet ned til det, en forlygte faktisk sender ud i luften.** Det var her, kasserne kom fra. Lyskeglens form er regnet som en andel af lygtens stærkeste punkt, og den har en bred "slette" ud til 45 grader til siden. På vejen er det rigtigt, men i luften betød det, at hver kubikmeter inde i hele den slette spredte lys, som om den stod midt i lygtens stærkeste stråle. Foran bussen blev luften derfor hvid over et stort område, og kanten af det, billedet stadig kunne rumme, var den skarpe, lige streg. Derfor så det ens ud foran din egen bus og foran AI-busser. Beams (forlygter og spots) sender nu 40 % ud i luften; gadelygter i deres armatur er uændrede. Tallet er fundet ved at se på billeder i tæt tåge med 0,15, 0,3, 0,5 og 1. Du kan prøve en anden værdi uden et nyt build med `OMSI_FOG_LAMP_GAIN`.
- Lygternes skær følges nu 24 felter ud ad synslinjen i stedet for 16. I kraftigt snevejr lyser luften 400 m ud, og de 16 felter nåede kun 280-400 m, så skæret fra en oplyst gade kunne stoppe brat langs en ret linje tværs over billedet.
- **Kasserne, tredje runde: lygtegitteret.** Det her er den egentlige årsag til de firkanter, der blev stående, også efter at skæret var skruet ned.
  Lygterne lægges i et gitter af felter på 25 × 25 m omkring kameraet, og hvert felt har plads til 32 lygter. Tågen slår en lygte op i det ene felt, hvor synslinjen passerer tættest forbi den — så en lygte, der ikke er med i det felt, er helt væk fra tågen netop der, og nabofeltet ved siden af har den med. Det er en lige kant på 25 m midt i luften, og det er kasserne.
  Når et felt var fyldt, gav den **svageste** lygte plads til en stærkere. Men alle forlygter er lige stærke, så de kunne ikke skilles: feltet beholdt dem, der nåede frem først, og det var forskellige lygter fra felt til felt. På en gade med et par busser og nogle biler er 32 pladser hurtigt brugt. Nu vægtes lygterne efter, hvor meget de lyser **det felt** op — styrken over afstanden — så den lygte, et felt afviser, er den, feltet alligevel næsten intet ser af, og nabofeltet afviser den samme. Dermed er der ingen kant.
  Oven i det: en lygte, der står inden for et par felter af øjet og rækker langt forbi det — altså bussens egne forlygter — slås nu op i synslinjens **første** felt, som er det felt, kameraet selv står i, og dermed det samme felt for hver eneste billedpunkt. Før lå det tætteste punkt på en forlygte halvanden meter foran øjet et par meter ud ad synslinjen, og det er et nyt felt for hver retning, øjet ser i; manglede lygten i et af dem, forsvandt dens lys i tågen for netop de billedpunkter. Hvilket felt der tager lygten, ændrer intet i regnestykket — strækningen, der lægges sammen over, er synslinjens egen.
  Med `OMSI_DEBUG_LIGHT_GRID=1` skriver spillet nu hver frame, hvor mange lygter der er, hvor mange der blev byttet ud af et fyldt felt, og hvor mange der blev afvist helt.
- **Gadelygternes skygger forsvinder og kommer igen glidende i stedet for på ét billede.** Det var dem, der fik vejen til at "blinke" lige dér, hvor der står lygtepæle.
  Kun **fire gadelygter ad gangen** har et skyggekort (`LAMP_SHADOWS`). Alle lygter inden for rækkevidde er kandidater, de fire bedste efter styrke over afstand får hver sit kort, og resten får ingen. I shaderen var det et rent ja eller nej: den frame en lygte mistede sin plads, forsvandt **alle** skygger under den på én gang — masten, buskene, kantstenen, en parkeret bil — og de kom lige så brat tilbage, når den vandt en plads igen. På vejbanen under en lygtepæl, som er det eneste sted, der overhovedet er sådan en skygge at tabe, læses det som skygger, der smutter ind og ud.
  Det er værst med indstillingen **"Skygger kastes af: alt"**, hvor der er mest at tabe, og næsten væk med OMSI's egne `[shadow]`-meshes — hvilket er sådan, fejlen blev fundet.
  Nu beholder en plads sin lygte, indtil lygten er tonet ud over 0,35 sekund, så dens kort stadig tegnes imens, og en lygte, der venter på en plads, tager den første, der bliver ledig. Udtoningen rider med i brøkdelen af lygtens nummer, så ingen anden struktur skal ændres. To tests holder øje med, at hverken den lygte, der går, eller den, der kommer, kan flytte sig mere end ét trin pr. billede, og at de tre, der beholder deres plads, slet ikke røres.
  `OMSI_LAMP_SHADOW_FADE=0` slår om med det samme igen, som før; et andet tal sætter tiden.
- **Lyset blinker og glimter ikke længere på vejen i nærheden af gadelygter.** Det ramte hele billedet: vejen, væggene, bygningerne og førerhusets instrumentbord på én gang.
  Om natten følger belysningen, hvor meget lygterne lyser dét, du kigger på. Det måles med **35 stråler** ud gennem synsfeltet ned på vejen — og målingen blev brugt, præcis som den stod, uden nogen udglatning. 35 punkter er meget få. Når du kører, fejer de hen over vejen, og hvert punkt falder enten ind i en lygtes pøl eller udenfor. Et punkt, der krydser kanten af en pøl, flytter hele målingen med et ryk.
  Målt på en nattur: tallet stod på **præcis nul i 1.828 af 14.568 billeder** (12,5 %), det hævede sig over kvarterets eget niveau i 18 korte perioder på omkring fire tiendedele sekund, og det **sprang 0,227 blændetrin fra ét billede til det næste**. I en videooptagelse af det samme målte jeg billedet springe 0,24 blændetrin på én frame — instrumentbordet faldt 71 %, vejen 22 %, mens de selvlysende displays kun faldt 4 % og HUD'en slet ikke rørte sig. Det er signaturen på, at hele belysningen tager et trin.
  Nu flyttes de 35 stråler en lille smule for hvert billede, så de ikke måler de samme steder om og om igen, og målingen glattes ud over 0,4 sekund — **logaritmisk**, som øjet selv tilvænner sig, for fra næsten mørke er et lille lineært skridt flere blændetrin. Oveni er der sat en grænse for, hvor hurtigt den må flytte sig: 1,5 blændetrin i sekundet, hvilket ved 100 billeder i sekundet er en halvfjerdsindstyvendedel blændetrin pr. billede. Det kan ikke ses som et spring, uanset hvad målingen gør. Tilsammen læser de 35 punkter som flere hundrede, uden at det koster mere.
  Og **målingen tages nu over øjets eget synsfelt, ikke kameraets.** De 35 stråler blev spredt ud over hele kameraets billedvinkel, så når du zoomede ud, fejede de hen over et helt andet stykke af gaden — og lyset i billedet ændrede sig af zoomet alene, mens du holdt stille og intet andet kunne ændre det. Nu holdes strålerne inden for 22,5 grader fra synsretningen, uanset hvor vidt kameraet står. Et kamera, der er smallere end det, beholder sit eget: man kan ikke tilvænne sig til noget, man ikke kan se.
  `OMSI_VIEW_LAMPS_EASE=0` slår det hele fra igen (målingen tages hel over kameraets felt, som før), og et andet tal sætter tiden.
- **Det grynede vævemønster i tåge og snevejr er væk.** Du så det tydeligst på bussens næsten sorte side, men det lå over alt mørkt i billedet.
  En forlygte rækker 225 m, så strækningen, skæret regnes over, er hele den kugle — men lygten lyser kun en smal kile af den. På de fleste synslinjer sidder lyset derfor i en lille stump af strækningen, og resten er mørk. Med 16 trin fordelt over det hele landede 15 af dem i mørket, og hele lyset faldt til det ene trin, der tilfældigvis ramte stumpen. Hvilket trin det blev, afgøres af synslinjens egen forskydning, og den er en hash af billedpunktets position — så nabopunkter fik fuldstændig urelaterede svar. Udregnet på forhånd svingede ét punkts resultat mellem **ingenting og 9 gange det rigtige** (spredning 60-229 %).
  Nu finder 17 billige prøver — kun lygtens form, ingen afstand, ingen spredningsfunktion — først hvor lygten overhovedet har noget at give, og de 16 rigtige trin lægges dér, én prøvebredde ud til hver side. Spredningen falder til **5-17 %**, og middelværdien bliver samtidig mere rigtig (0,99 af det eksakte svar mod 0,93-0,98), fordi intet går tabt: uden for stumpen er der ingenting at integrere. Finder prøverne ingenting, beholdes hele strækningen som før.
- **Lyskeglen i tågen har ikke længere en knivskarp kant.** OMSI's kegle (`light_cone.bmp`) blev skåret ud af sin firkant med et rent ja/nej, så den endte i en bue ved sin rækkevidde og i to snorlige streger langs sin yderste vinkel — en hård kile af lys ud af hver lygte. Tåge har ingen kanter, så keglen har ikke længere nogen: den går ud over den sidste femtedel af sin rækkevidde og over den yderste tredjedel af sin spredning. (Det var ikke det samme sted i koden som skæret ovenfor, og derfor rørte hverken `OMSI_NO_FOG_LAMPS` eller `OMSI_FOG_LAMP_GAIN` det.)

### Multiplayer
- **Spillerlisten i hjørnet:** øverst står **Online: koden** til sessionen. Hos gæsterne står **Online: i spillet hos Mathias**. Under den står én linje pr. spiller, også dig selv, i den rækkefølge I kom ind: **Spiller 2: bjorn - Linje 135 & Tur 135S05 - Fra kl. 14:12 - 22:40**, eller **fri kørsel**, når en spiller ikke har nogen vagt. Ny indstilling **Online: alle spillere i hjørnet** (slået til fra start; i launcheren ved chatten og i spillets menu). Slået fra står der kun den korte linje som før: "Online: 1 anden spiller  KODE".
- Beskederne om spillere i chatten er på dansk: **Spiller bjorn er kommet ind med Urbino_18_…**, **Spiller bjorn har forladt spillet** (også når forbindelsen er tabt), **Spiller bjorn er tilbage**.
- **Chatten forsvinder af sig selv:** hver linje toner ud 30 sekunder efter, den kom. Holder du musen over chatten eller trykker / for at skrive, kommer hele historikken frem igen.
- Loggen fyldes ikke længere op, når et andet program på computeren (fx Steam, hvis standardport er 27015) banker på værtens port hele tiden. Det skrives kun de første gange og derefter ved 10, 100, 1000 osv.
- På minimappet og bykortet står de andre spilleres **linje foran deres navn**, fx **123 - Bjorn**, eller **Ikke i rute - Bjorn**, når de ikke har nogen linje. Går de til fods eller kører med i en andens bus, står kun navnet.
- Destinationsskiltet (matrix) på de andre spilleres busser viser nu linje og destination. Før forblev skiltet ofte sort, fordi bussens script troede, at det allerede havde tegnet skiltet.
- Skiltet på de andre spilleres busser bruger nu den rigtige skrift. Før kunne destinationen stå som mærkelige tegn (eller slet ikke), fordi skrifttypernes numre blev kopieret fra den anden spillers computer, hvor de ligger i en anden rækkefølge.
- Destinationsskiltet på de andre spilleres busser tegnes nu af bussen selv på din computer ud fra deres linje og destination, som på en AI-bus. Før blev skiltets egne mellemregninger kopieret fra den anden computer, og på fx Solaris Urbino IV (Iwo) blev skiltet tomt eller viste kun prikker.
- Skiltet på de andre spilleres busser (fx Solaris) kunne stå tomt, når bussen dukkede op. Destinationen blev sat, før bussens script var startet, og blev aldrig sat igen. Nu sættes den igen efter 1, 5 og 20 sekunder. Har din HOF-fil for bussen ikke deres destination, leder spillet bussens andre HOF-filer igennem efter en, der har den. Loggen skriver det, hvis ingen af dem har den. Når en anden spillers bus kommer **inden for 200 m** af dig, tegnes dens skilt igen hos dig, og spillets log skriver hvem, hvilken linje og destination og hvilken HOF-fil. Sender deres spil ingen destination, står det også i loggen. Det sker igen, når bussen har været mere end 250 m væk. Alle spillere skal have den nye version for at se det hos sig.
- Passagerer i de andre spilleres busser sidder nu ned på sæderne. Før stod de oprejst oven på sædet.
- Passagerer, der stiger på en anden spillers bus, beholder den destination og turlængde, de havde hos værten, i stedet for at få en tilfældig. En **gruppe** følges ad: beder bussen om én af dem, får den hele gruppen, og hos den anden spiller sidder de sammen, én køber billetter til alle, og de stiger af sammen. Begge spillere skal have den nye version.
- Ved et stoppested får en anden spillers bus nu kun de ventende, der skal med dens linje (eller ikke venter på en bestemt linje), ligesom værtens egen bus. Før fik den alle, der ventede ved stoppestedet, også dem til andre linjer, så den anden spillers bus blev fyldt, mens værtens fik langt færre.

### Passagerer
- **Passagerer der løber efter bussen** (fra openOMSI PR #2073 af JDCustard, MIT-licens, i vores egen, lettere udgave). Kun ved **spillernes busser** (din egen og de andre spilleres over LAN), og kun når bussen **holder stille** ved et stoppested nær dig. Så kommer der en gang imellem en, der er sent på den, og **løber** efter bussen: med længere skridt, bøjede knæ og albuer, lænet fremover. Det er en, der går på fortovet i nærheden, eller en, der kommer bagfra, uden for synsvidde, så du ser dem komme i spejlet. Kører du fra dem, bliver de stående et øjeblik og går videre. Ved en anden spillers bus løber de hen til stoppestedet, hvor hans spil tager dem med. AI-busser får kun løbere, når de holder ved stoppestedet med **åbne døre**, og en spiller (værten eller en anden spiller over LAN) er **højst 50 m** fra bussen. Så venter AI-bussen op til 12 sekunder på løberen.
  Ny indstilling **Passagerer der løber efter bussen**: Fra, En gang imellem (5 %, standard), Tit (15 %) eller Meget tit (30 %) pr. stop. Findes i launcheren under Gameplay → Passagerer og i spillets menu. Spillet tjekker det kun 4 gange i sekundet, og kun hvis en spillers bus holder stille. Ellers laves der intet.
- Ny indstilling **Mennesker på gaden**: Ingen, Få, Normal, Mange eller Rigtig mange. Den bestemmer, hvor mange der går på fortovene, uafhængigt af antallet af passagerer ved stoppestederne. Findes i launcheren under Gameplay → Passagerer og i spillets menu, og virker med det samme. Skruer du ned, forsvinder de overskydende, når du ikke kan se dem.
- I din egen bus sætter passagererne sig, hvis der er en ledig plads. På din vagt står 7 ud af 10 dog op, hvis de kun skal under **1 km**. Er den ønskede slags plads optaget, tager de den anden. Ændringen kommer fra openOMSI PR #1394.
- Ny indstilling **Passagerernes lyst til at sidde**: Fra, Lidt, Normal (standard), Meget eller Rigtig meget. Findes i launcheren under Gameplay → Passagerer og i spillets menu. **Fra** giver en tilfældig ledig plads som i OMSI 2. Ellers vælger passagererne plads som rigtige mennesker:
  - De tager helst en **vinduesplads** i et **tomt dobbeltsæde** og sætter sig først ved siden af en fremmed, når der ikke er andet.
  - **Ældre passagerer** (65 år og derover efter personens fil, fx den ældre dame med gråt hår og beige cardigan) går **ofte** efter **pilotsædet**: sædet på den anden side af gangen fra føreren, lidt bag førerpladsen. Det virker også, hvis rattet sidder i højre side. De andre lader det stå, så længe de har et andet.
  - Cirka hver femte venter og rejser i en **gruppe** på 2-4 med samme destination: to tager et dobbeltsæde, tre eller fire en "firer" (to dobbeltsæder over for hinanden), ellers sidder eller står de tæt ved hinanden. Har kortet børnefigurer, kan det også være en voksen med 1-2 børn. Den første i gruppen køber **billetter til hele gruppen** hos føreren, én efter én (fx en voksenbillet og to børnebilletter), og de stiger af sammen.
  - I din bus sætter en, der står og skal et stykke endnu, sig på en plads, der bliver ledig, mens bussen kører (én ad gangen).
  - **Nu og da** (højst hvert 2½ minut) rejser en yngre passager sig for en ældre, der ikke fik en siddeplads.
  - Jo større lyst til at sidde, jo flere sætter sig på korte ture, og jo oftere flytter de, der står, hen på en ledig plads.
- **Kun busser, der faktisk kommer:** folk ved et stoppested vil kun med en bus, der afgår derfra inden for de næste **20 minutter** (før 15). Ingen står og venter kl. 11 på en bus kl. 23. Stiller du klokken mere end 10 minutter, går dem, hvis bus ikke længere kommer snart, deres vej.
- **Stoppestedets "exit"-tal bliver respekteret:** hvor mange der skal af ved et stop, læses nu fra stoppestedsobjektet via køreplanens navn på stoppet. Før faldt det ofte tilbage til et standardtal, når navnet på objektet var anderledes. Et stop med **0** (fx et servicestop) bliver aldrig valgt som destination. Et stop, hvor alle destinationer har 0, har ingen ventende.
- **Fodgængere sidder ikke fast:** folk til fods støder ikke længere ind i sceneriet (lygtepæle, hegn og tilfældige objekter), hvor de før kunne sidde fast. De holder sig stadig fri af køretøjerne. Det sparer også arbejde i hvert billede.
- **Ingen skridt tilbage før udgangen:** passagerer, der venter ved en lukket udgang, træder frem, når den åbner. Før gik de ofte et punkt tilbage og vendte om. En, der allerede er på vej mod udgangen, går videre fremad i stedet for tilbage til punktet bag sig.

### Køreplanens busser
- AI-busser **slukker motoren** ved endestationen, 6 sekunder efter turen er slut, mens de venter på næste tur. Før kørte motoren videre.
- Når du **stiller klokken**, forsvinder AI-busserne og kommer igen, hvor køreplanen har dem på det nye tidspunkt. Det sker, når du har sluppet skyderen eller knapperne et øjeblik. Små ændringer lægges nu sammen, så fem gange +1 minut også tæller, og en skyder trukket over flere timer giver kun én omstart.

### Spillet
- **Skift køretøj** (Alt-menuen) viser nu en liste over de køretøjer, der står i verden, med det nærmeste først og afstanden. Det, du vælger, overtager du. Skift fra Esc-menuen virker nu også, når du går til fods: du sættes ved rattet i stedet for at blive stående.
- **Teleport** (til et punkt på kortet eller til et startpunkt) virker kun, når du sidder i køretøjet, da det er køretøjet, der flyttes.
- Knapperne **Solopgang** og **Solnedgang** under Verden → Tid stiller klokken til den rigtige solopgang og solnedgang for dagen og kortets placering (fra kortets timezone.txt) i stedet for 06:00 og 18:00 hele året.

### Launcher
- Når du vælger en bus, vælges en **tilfældig bemaling** (standardbemalingen er en af dem), som vises og køres med, i stedet for altid standardbemalingen.
- Køretøjers **beskrivelser** bliver ikke længere skåret af ved en linje i firkantede parenteser (fx "[Tekniske data]"), og beskrivelser gemt som UTF-8 vises med de rigtige bogstaver (æ, ø, å).

### Server
- På en dedikeret server følger køreplanens busser nu serverens ur, dens hastighed og en admins ændring af klokken. Før kørte de videre fra starttidspunktet, uanset hvad klokken blev sat til (openOMSI PR #1970 af veryinsanee).
- På en **dedikeret server** venter der nu folk ved alle stoppesteder, efterhånden som deres afgange nærmer sig. Før kom der kun passagerer ved de stoppesteder, der havde en afgang i serverens første kvarter, og resten stod tomme hele aftenen. Ændringen kommer fra openOMSI PR #1964 (veryinsanee). Når du er vært fra dit eget spil, virkede det allerede.

### Rat og controllere
- Uden rat eller controller tilsluttet leder spillet nu efter nye hvert 5. sekund i det første minut og derefter én gang i minuttet, i stedet for hvert 5. sekund hele tiden. Et rat, der sættes i, findes stadig med det samme. Loggen skriver kun, når der findes noget nyt.

### Rettelser fra openOMSI
- **Bussignaler:** et særligt bussignal kunne vise rødt og grønt på samme tid, eller gå i stå på rødt ved starten af sin cyklus, så busserne holdt for evigt (fx på Grand Paris-Moulon og Saint Servan). Begge dele er rettet (openOMSI PR #2264 og #2265 af YanisK33, issue #2266). I vores udgave sammenlignes lampernes navne uden at lave en ny kopi for hver lampe i hvert billede.
- **Hurtigmenu på Alt (som i OMSI 2):** et kort tryk på Alt alene åbner 12 felter nederst til højre: placer, skift, fjern og nulstil køretøj, køreplan, destination, reparation, tank op/vask, rutepile, tid, vejr og statistik (turoversigt og personalemappe). Spillet kører videre imens. Alt eller Esc lukker menuen, og det gør et klik ved siden af også. Alt+Enter og Alt+Tab åbner den ikke. Når menuen er lukket, koster den intet (openOMSI 0.2.28, commit 3ecd79b4 af turbo-devv).
- **Til fods under et tag:** står du op under en overdækning eller i en indendørs busterminal, ender du ikke længere oppe på taget. Du står på gulvet, og kameraet følger med (openOMSI PR #2201 af KYChung1287).
- **Køreplanens busser skifter fra gang til gang:** hver tur fik før den samme bus fra garagen i hver session. Nu trækkes bussen tilfældigt pr. session, som i OMSI, og ens for alle spillere i et LAN-spil (openOMSI commit 9daf66d).
- **HOF-filer til busserne:** når du installerer et mod-kort, kopierer launcheren kortets HOF-filer (fra `ailists.cfg`) ind i de busmapper, der allerede har HOF-filer. Intet bliver overskrevet, og OMSI-mappen bliver ikke rørt. Under Mods er der også en knap til at tilføje HOF-filer til busserne manuelt (openOMSI PR #2082 af dxox57).

### Opdateringer
- openOMSI DK opdaterer sig selv fra denne fork og ikke fra den officielle openOMSI, så ændringerne bliver.
- Der bygges kun til almindelige Windows-pc'er (x64), ikke længere til Windows på ARM (Snapdragon-bærbare).
