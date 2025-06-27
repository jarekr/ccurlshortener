
import requests

URLS="http://localhost:8080/e/CtKY5TfhONM= http://localhost:8080/e/hatCLxyKmXY= http://localhost:8080/e/ES8ZhVeI1Hc= http://localhost:8080/e/93kwOTFrVW0= http://localhost:8080/e/LaEbsb3D10g= http://localhost:8080/e/z1IPeJCESpI= http://localhost:8080/links"

while true; do
for url in $URLS; do
  echo $url
  time curl -i $url
  echo
done
done
