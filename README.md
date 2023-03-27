Build instructions: (run these commands)

rustup install 1.30.0
cargo install --locked trunk
trunk serve --release                (build server)

Project archecture
    main - used to coordinate build options
        app - interactable application
            app_widgets - widgets used in the interface
            app_pages - use widgets to build the pages in the app
            cube_painter - WebGL interface for painting cubes
            data - processed data into verticies
            tree - input file into processed data
            ui_helper - ui functions for interactivity
        shaders - simple triangle shaders for rendering the cubes

Data flow as follows
input files -> Trace -> Tree -> [verticies, indicies, colors] -> paint to WebGL -> user input  


Future optimization: 
Speed up tree regeneration by storing a vector of time ranges each trace falls under
List to speed up processing: Vec<(Trace, RangeSet<TraceRanges>)>'
RangeSet: some data sctructure TBD
TraceRanges: start end and value. 
This differes from current implementation as the RangeSets would be a specitalized set that only show you traces with the range.




TODO
show and copy text from inspector view 

drag and drop files

program profiling technqiue of per stacktrace over time display upon selecting a node 

color stacktraces hue based on thier overall trend tword a metric

seperate flamegraphs more and add a slider to traverse between them

on high division splits, cap divisions at ~10 pixels wide and add a scroll bar on the overveiw
