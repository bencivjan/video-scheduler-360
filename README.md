# Video-Scheduler-360

In the rapidly evolving field of multimedia technology, 360°
videos have emerged as a groundbreaking format that offers
immersive experiences. However, it poses significant challenges for video servers, such as high bandwidth demand,
large video size, and heterogeneous user requirements. I propose two novel scheduling policies for 360°
video servers that aim to optimize the quality of experience
(QoE) for users while minimizing the server cost. The instructor/observer policy takes into account different user
roles to more effectively prioritize content distribution. The
field-of-view policy leverages the viewing angle of the user
to determine which threads to prioritize based on whether
"content of interest" is being viewed.


I evaluate these algorithms by implementing a HTTP video
server and user-space thread scheduler to conduct a series of
experiments to evaluate average thread runtime. I conclude
that if clients seek to download the video as fast as possible,
it is beneficial to choose a large base time quantum. The
field-of-view scheduling policy works best when the video’s
horizon is of interest to many users. But when seeking to
reduce the download time of high-priority clients, the instructor/observer
policy with a high instructor multiplier is
the best option.

View the full report [here](ben_civjan_cs537_final_report.pdf).
