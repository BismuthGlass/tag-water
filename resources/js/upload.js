function upload_files(input_field) {
    console.log(input_field.files);
    const form_data = new FormData();
    for (let i = 0; i < input_field.files.length; ++i) {
        form_data.append(`files[${i}]`, input_field.files[i]);
    }
    console.log(form_data);

    fetch("/upload/new", { 
        method: "POST",
        body: form_data,
    }).then((response) => console.log(response));
}

function open_files() {
    document.getElementById('file-upload-input').click();
}

function toggle_editor() {
    document.getElementById('script-editor').classList.toggle("hidden");
}