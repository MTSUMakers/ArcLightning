// Insert URLs to videos here
var videos = ['demo_videos/Demo1_dbfz.webm', 
              'demo_videos/Demo2_ddr.webm', 
              'demo_videos/Demo3_tekken.webm']
videoPlayer = document.getElementById("bgvid");
video_count = 0;
function demoRun() {
    video_count++;
    if (video_count == videos.length) video_count = 0;
    var nextVideo = videos[video_count];
    videoPlayer.src = nextVideo;
    videoPlayer.play();
};