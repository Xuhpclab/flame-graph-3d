<!-- ABOUT THE PROJECT -->
## About The Project

Flamegraphs are standard profiling tools in both industry and academia used to visualize large amounts of performance data. 
Existing flamegraphs are two dimensional, which limits them to showing one metric at once. 
This project, EasyFlame, is a performance analysis framework based on 3D flamegraphs.
EasyFlame extends existing approaches through its inclusion of an extra dimension that shows how a metric changing across time.

## Using without download

1: Visit https://www.xperflab.org/web/
2: nagivate to '3D View' and then 'Example 3D view' 
3: Application will open. Open collapsable open panels to utilize multimetric analysis features. 
Note: Due to cookie cache, visiting in an incognito browser will ensure the latest update is loaded. 

## Installation and build 

Build instructions: (run these commands)

0. clone repo and enter the 'rustBuild' folder 
1. rustup install 1.30.0
2. cargo install --locked trunk
3. trunk serve --release
4. navigate to http://127.0.0.1:8080/web in an incognito browser window 

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Project archecture

    main - used to coordinate build options
        app - interactable application
            app_widgets - widgets used in the interface
            app_pages - use widgets to build the pages in the app
            cube_painter - WebGL interface for painting cubes
            data - processed data into verticies
            tree - input file into processed data
            ui_helper - ui functions for interactivity
        shaders - simple triangle shaders for rendering the cubes

## Roadmap

        show and copy text from inspector view 
        toggle to display all text for nodes in inspector 
        drag and drop files to load data
        generate and display stacktrace upon selecting a node
        option to color stacktraces hue based on thier overall trend tword a metric
        seperate flamegraphs more and add a slider to traverse between them
        on high division splits, cap divisions at ~10 pixels wide and add a scroll bar on the overveiw

<!-- LICENSE -->
## License

 TBD


<!-- CONTACT -->
## Contact

Alex Boots - alexjboots@gmail.com


<p align="right">(<a href="#readme-top">back to top</a>)</p>













