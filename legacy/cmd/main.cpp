#include <opencv2/opencv.hpp>
#include <string>




using namespace cv;
using namespace std;

int main()
{


    string texte ; // text pour la video
    string Yes = "y"; // variable de condition pour modification
    string Non = "n"; // variable de condition pour modification
    string Oui;

    int vitesseDefilement = 4;

    cout << "Vous devez entrer votre text (accents non pris en compte):\n";

    getline(cin, texte);

    cout << "Voulez vous changer la vitesse de deffilement ? :\n";
    cout << "Oui = y | Non = n \n" ;

    cin >> Oui ;

    if (Yes == Oui)
    {
        cout << "Entrer le valeur de la vitesse de deffilement :\n";

        cout << "Lente  = 2 :\n";
        cout << "Normal = 4 :\n";
        cout << "Rapide = 6 :\n";

        cout << "Valeur = ";

        cin >> vitesseDefilement ;

        cout << "\n";

    }

    int espaceEntreTexte = 12;
    int epaisseurPolice = 6;
    
    Scalar couleurTexte(0, 0, 0); // couleur du text noir

    int fontFace = FONT_HERSHEY_SIMPLEX;
    double fontScale = 2.5;

    int largeurImage = 1280; //définiton horizontal ←→
    int positionX = largeurImage;
    int hauteurImage = 180;    //définition vertical ↑↓

    int fps = 60; // Images par seconde

    cout << "Voulez vous changer les parametres videos ? (1280x180 par defaut) :\n";
    cout << "Oui = y | Non = n \n";

    cin >> Oui;

    if (Yes == Oui)
    {
        cout << "Entrer le valeur horizontal (vous modifirer la valeur '1280' pour une definition de 1280x180) :\n";
        cin >> largeurImage;
        cout << "Entrer le valeur vertical (vous modifirer la valeur '180' pour une definition de 1280x180) :\n";
        cin >> hauteurImage;
        cout << "\n";

    }

    string nomdufichier; // nom du ficher de sortie 
    string extensionvideo = ".mp4"; // format du ficher de sortie
    cout << "Entrer le nom du fichier video de sortie :\n";
    cin >> nomdufichier; 


    cout << "\n";
    cout << " -----------------------------------------------------------------------------------\n ";
    cout << "Video en cours de traitement veuillez patienter.\n ";
    cout << "Cette fenetre ce fermera une fois la video prete.\n ";
    cout << "Pensez a verifier le resultat.\n ";
    cout << "-----------------------------------------------------------------------------------\n ";

    cout << "\n";
    cout << "\n";
    cout << "\n";

    // création de la video 

    nomdufichier.append(extensionvideo);

    VideoWriter video( nomdufichier, VideoWriter::fourcc('M', 'J', 'P', 'G'), fps, Size(largeurImage, hauteurImage));

    if (!video.isOpened())
    {
        cout << "Impossible de créer la vidéo." << endl;
        return -1;
    }



    Mat image(hauteurImage, largeurImage, CV_8UC3, Scalar(255, 255, 255)); // couleur du fond 

    int texteLargeur = getTextSize(texte, fontFace, fontScale, epaisseurPolice, nullptr).width;
    int texteHauteur = getTextSize(texte, fontFace, fontScale, epaisseurPolice, nullptr).height;

    int positionY = (hauteurImage + texteHauteur) / 2;

    bool texteVisible = true;

    while (texteVisible) // Algoritme pour que la longeur de la video face 1 defillement de droite vers la gauche.
    {
        image.setTo(Scalar(255, 255, 255)); // Effacer l'image précédente

        putText(image, texte, Point(positionX, positionY), fontFace, fontScale, couleurTexte, epaisseurPolice);

        video.write(image);

        positionX -= vitesseDefilement;

        int texteLargeur = getTextSize(texte, fontFace, fontScale, epaisseurPolice, nullptr).width;
        if (positionX < -texteLargeur)
        {
            texteVisible = false;
        }

        if (waitKey(1) == 27)
            break;
    }

    video.release();


    return 0;
}
