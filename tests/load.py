
import requests

urls = [
"http://localhost:8080/e/CtKY5TfhONM=",
"http://localhost:8080/e/hatCLxyKmXY=",
"http://localhost:8080/e/ES8ZhVeI1Hc=",
"http://localhost:8080/e/93kwOTFrVW0=",
"http://localhost:8080/e/LaEbsb3D10g=",
"http://localhost:8080/e/z1IPeJCESpI=",
]


for x in range(100):
    for url in urls:
        response = requests.get(url)
        print(f"got response {response.status_code}:{response.reason}")
