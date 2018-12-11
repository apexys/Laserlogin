let getCookieValue = (a) => {
    let b = document.cookie.match('(^|;)\\s*' + a + '\\s*=\\s*([^;]+)');
    return b ? decodeURIComponent(b.pop()) : '';
}

let message = getCookieValue('message');
if(message.startsWith('update')){
    gid('message').innerText = message.substring(7);
    gid('flash-container').style.display = 'block';
    gid('flash-container').style.opacity = 1;
    gid('flash').classList.add('bg-blue');
    f('/clear_message');
}else if(message.startsWith('error')){
    gid('message').innerText = message.substring(6);
    gid('flash-container').style.display = 'block';
    gid('flash-container').style.opacity = 1;
    gid('flash').classList.add('bg-red');
    f('/clear_message');
}

setTimeout(()=> {
    gid('flash-container').style.opacity = 0;
}, 2500);

setTimeout(()=> {
    gid('flash-container').style.display = 'none';
}, 3000);