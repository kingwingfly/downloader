import ConfigForm from "@/components/config-form";
import HomeBtn from "@/components/home-btn";

export default function ConfigPage() {
    return (
        <div className="flex flex-col place-items-center gap-4">
            <ConfigForm />
            <HomeBtn />
        </div>
    )
}