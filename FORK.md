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

### Lys
- Forlygterne fra en bus eller bil bag dig (andre spillere og AI) lyser ikke længere igennem din bus ind på forruden, instrumentbrættet og kabinen, når du sidder i førerhuset (grafik: Enhanced og Enhanced+).
- Det gælder nu alle lygter på andre køretøjer (også deres positionslys og de forenklede lygter), og også med grafikken Vanilla og Vanilla+.
- Din bus kaster nu skygge for lyset fra køretøjer bag dig: deres forlygter lyser ikke længere vejen, bilerne og husene **foran** din bus op, som om din bus ikke var der (det så ud som forlygter i forruden). Gælder alle grafikindstillinger, når du sidder i bussen.
- Hurtigere: skyggen regnes nu kun ud for lygter, hvis lys kan nå forbi din bus. Før blev hvert billedpunkt tjekket mod alle bilers lygter i byen, hvilket kostede om natten.

### Multiplayer
- **Spillerlisten i hjørnet:** øverst står **Online: koden** til sessionen. Hos gæsterne står **Online: i spillet hos Mathias**. Under den står én linje pr. spiller, også dig selv, i den rækkefølge I kom ind: **Spiller 2: bjorn - Linje 135 & Tur 135S05 - Fra kl. 14:12 - 22:40**, eller **fri kørsel**, når en spiller ikke har nogen vagt. Ny indstilling **Online: alle spillere i hjørnet** (slået til fra start; i launcheren ved chatten og i spillets menu). Slået fra står der kun den korte linje som før: "Online: 1 anden spiller  KODE".
- Beskederne om spillere i chatten er på dansk: **Spiller bjorn er kommet ind med Urbino_18_…**, **Spiller bjorn har forladt spillet** (også når forbindelsen er tabt), **Spiller bjorn er tilbage**.
- **Chatten forsvinder af sig selv:** hver linje toner ud 30 sekunder efter, den kom. Holder du musen over chatten eller trykker / for at skrive, kommer hele historikken frem igen.
- Loggen fyldes ikke længere op, når et andet program på computeren (fx Steam, hvis standardport er 27015) banker på værtens port hele tiden. Det skrives kun de første gange og derefter ved 10, 100, 1000 osv.
- På minimappet og bykortet står de andre spilleres **linje foran deres navn**, fx **123 - Bjorn**, eller **Ikke i rute - Bjorn**, når de ikke har nogen linje. Går de til fods eller kører med i en andens bus, står kun navnet.
- Destinationsskiltet (matrix) på de andre spilleres busser viser nu linje og destination. Før forblev skiltet ofte sort, fordi bussens script troede, at det allerede havde tegnet skiltet.
- Skiltet på de andre spilleres busser bruger nu den rigtige skrift. Før kunne destinationen stå som mærkelige tegn (eller slet ikke), fordi skrifttypernes numre blev kopieret fra den anden spillers computer, hvor de ligger i en anden rækkefølge.
- Destinationsskiltet på de andre spilleres busser tegnes nu af bussen selv på din computer ud fra deres linje og destination, som på en AI-bus. Før blev skiltets egne mellemregninger kopieret fra den anden computer, og på fx Solaris Urbino IV (Iwo) blev skiltet tomt eller viste kun prikker.
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
