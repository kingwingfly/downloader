import httpx
import asyncio

headers = {
    # "Accept": "*/*",
    # "Accept-Encoding": "identity",
    # "Accept-Language": "en-US,en;q=0.9",
    # "Connection": "keep-alive",
    # "Host": "xy123x184x20x101xy.mcdn.bilivideo.cn:4483",
    # "Host": "upos-sz-mirrorcos.bilivideo.com",
    # "Origin": "https://www.bilibili.com",
    "Range": "bytes=0-8000",
    "Referer": "https://www.bilibili.com/",
    # "Pragma": "no-cache",
    # "Sec-Fetch-Dest": "empty",
    # "Sec-Fetch-Mode": "cors",
    # "Sec-Fetch-Site": "cross-site",
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15",
}

# url = "https://xy123x184x20x101xy.mcdn.bilivideo.cn:4483/upgcxcode/66/77/1049107766/1049107766-1-30280.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1698224555&gen=playurlv2&os=mcdn&oi=3736210139&trid=00002cf62a7aeb0a47169126288963d78aafu&mid=32280488&platform=pc&upsig=673aa5d1479be2f2e0ad0cdb9e5bafc2&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&mcdnid=18000574&bvc=vod&nettype=0&orderid=0,3&buvid=4B8B34C3-1049-CA66-CD1F-300F94C11BFB45796infoc&build=0&f=u_0_0&agrr=1&bw=30625&logo=A0020000"
# url = "https://upos-sz-mirror08c.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30112.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&amp;uipk=5&amp;nbs=1&amp;deadline=1698230971&amp;gen=playurlv2&amp;os=08cbv&amp;oi=0&amp;trid=c970410f70bc4c4aa9cce3f7d5282ec7u&amp;mid=32280488&amp;platform=pc&amp;upsig=4e6b26f584cd50c8795b0c140fad49d8&amp;uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&amp;bvc=vod&amp;nettype=0&amp;orderid=0,3&amp;buvid=&amp;build=0&amp;f=u_0_0&amp;agrr=1&amp;bw=669180&amp;logo=80000000"
url = "https://upos-sz-mirrorcos.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30280.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&amp;uipk=5&amp;nbs=1&amp;deadline=1698230971&amp;gen=playurlv2&amp;os=cosbv&amp;oi=0&amp;trid=c970410f70bc4c4aa9cce3f7d5282ec7u&amp;mid=32280488&amp;platform=pc&amp;upsig=9f15cb0997298e1ad7086d647eff7795&amp;uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&amp;bvc=vod&amp;nettype=0&amp;orderid=0,3&amp;buvid=&amp;build=0&amp;f=u_0_0&amp;agrr=1&amp;bw=30625&amp;logo=80000000"
url = "https://cn-tj-cu-01-09.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30112.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1698232493&gen=playurlv2&os=bcache&oi=0&trid=0000e4541fe0d5e24961b19dc212d3aae8ecu&mid=32280488&platform=pc&upsig=c57990511f16d541759a9c49fd2aaceb&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&cdnid=87209&bvc=vod&nettype=0&orderid=0,3&buvid=4B8B34C3-1049-CA66-CD1F-300F94C11BFB45796infoc&build=0&f=u_0_0&agrr=1&bw=669180&logo=80000000"


async def main():
    async with httpx.AsyncClient(headers=headers) as client:
        resp = await client.get(url)
        print(resp.status_code)
        # assert resp.status_code == 203


if __name__ == "__main__":
    asyncio.run(main())
