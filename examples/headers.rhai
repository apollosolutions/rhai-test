fn rename_header(request){
    let header_map = #{
        "assetid": "original_assetid"
    };

    for header in header_map.keys(){
        if header in request.headers {
            let renamed = header_map[header];
            request.subgraph.headers[renamed] = request.headers[header];
        }
    }
}