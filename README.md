# rs-repo-manager

Simple project for simply taking care of my docker repo's on my raspberry pi.
I could have written this application in any language (probably should have done so in C#)
But it's been a while since I last had a Rust project


this project has a simple problem to solve: the way my workflow is working right now, whenever I git push to a repo I have to then manually ssh into my raspberry pi and manually git pull -> docker compose down -> docker compose up.
That works well when I'm only developing one feature and calling it a day but let's picture a situation where I fix something important on the main branch then I want to move on with developing a feature in another branch, I would then need to stop coding, breaking the flow of the coding session just to deploy it just to then come back and keep developing.

Why not just use a CI/CD pipeline ?
That's actually a good question, I'd rather use a very conservative firewall rules, only allowing for ssh on local ip's than to publish my pi on the internet and hope that my firewall/protection rules are protecting me from every possible attacker.


The idea of this project is to be used in a linux cron or in a sleep based infinite loop (cron preferably though), the idea is very simple:
every service has a directory.
there are 3 major commands: git pull, docker compose down, docker compose up -d --build.
I want to give the possibility of the user configuring commands to run in between them and after the end, so the execution flow is

(0) create a list of directories to check
(1) cd into the directory
(2) git pull
(2.1) if no changes were found, go to the other directories
(3) run optional command 1
(4) docker compose down (optional)
(5) run optional command 2
(6) docker compose up -d --build
(7) run optional command 3
(8) move on to the next directory in the list.

should be this simple to model and this simple to execute.