from subprocess import call

urls = [
"http://localhost:8000/e/CtKY5TfhONM=",
"http://localhost:8000/e/lvKIq6i5O0s=",
"http://localhost:8000/e/ARtv-f8iENM=",
"http://localhost:8000/e/e7BDQlkDeok=",
"http://localhost:8000/e/wuTpmLjPu_Q=",
"http://localhost:8000/e/ES8ZhVeI1Hc=",
"http://localhost:8000/e/hatCLxyKmXY=",
"http://localhost:8000/e/8xko3dGd9d8=",
]

for url in urls:
    call(["curl", "-i", "-X", "GET", url])

print("all done")
